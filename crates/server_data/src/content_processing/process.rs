use std::collections::{HashMap, VecDeque};

use model::{
    AccountId, AccountIdDb, AccountIdInternal, ContentProcessingState,
    ContentProcessingStateChanged, ContentProcessingStateInternal, ContentSlot,
};
use model_server_data::NewContentParams;
use tokio::sync::{Notify, OwnedMutexGuard, RwLock};

use super::{ProcessingPhase, UploadPermit};
use crate::{event::EventManagerWithCacheReference, file::utils::TmpContentFile};

#[derive(Debug)]
pub struct ContentProcessingReceiver(pub std::sync::Arc<Notify>);

pub struct ProcessManagerData {
    event_queue: std::sync::Arc<Notify>,
    data: RwLock<Data>,
}

impl ProcessManagerData {
    pub fn new() -> (Self, ContentProcessingReceiver) {
        let notify = std::sync::Arc::new(Notify::new());
        (
            Self {
                event_queue: notify.clone(),
                data: RwLock::new(Data::default()),
            },
            ContentProcessingReceiver(notify),
        )
    }

    pub fn data(&self) -> &RwLock<Data> {
        &self.data
    }

    /// Queue new content for account-level processing.
    pub async fn queue_new_content(
        &self,
        content_owner: AccountIdInternal,
        slot: ContentSlot,
        upload_info: UploadInfo,
        new_content_params: NewContentParams,
        mut processing_phase_lock: OwnedMutexGuard<ProcessingPhase>,
    ) {
        let UploadInfo {
            tmp_raw_img,
            tmp_img,
            upload_permit,
        } = upload_info;
        *processing_phase_lock = ProcessingPhase::Processing;
        drop(processing_phase_lock);
        drop(upload_permit);

        let mut write = self.data.write().await;

        let key = content_owner.id;
        write.queue.push_back(key);
        let state = ProcessingState {
            content_owner,
            slot,
            processing_state: ContentProcessingStateInternal::InQueue {
                wait_queue_position: write.queue.len().try_into().unwrap_or(i64::MAX),
            },
            tmp_img,
            tmp_raw_img,
            new_content_params,
        };
        write.processing_states.insert(key, state);

        self.event_queue.notify_one();
    }

    pub async fn pop_from_queue(
        &self,
        events: EventManagerWithCacheReference<'_>,
    ) -> Option<ProcessingState> {
        const CHECKPOINT_POSITIONS: [usize; 29] = [
            5, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 200, 300, 400, 500, 600, 700, 800, 900,
            1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000,
        ];

        let mut write = self.data.write().await;
        let processing_id = write.queue.pop_front()?;

        let mut queue_position_update_events: [Option<(AccountId, ContentProcessingStateChanged)>;
            CHECKPOINT_POSITIONS.len()] = std::array::from_fn(|_| None);

        for (queue_position, update_event) in CHECKPOINT_POSITIONS
            .into_iter()
            .zip(queue_position_update_events.iter_mut())
        {
            let Some(processing_id_in_queue) = write.queue.get(queue_position - 1).copied() else {
                break;
            };

            if let Some(state) = write.processing_states.get_mut(&processing_id_in_queue)
                && let ContentProcessingStateInternal::InQueue {
                    wait_queue_position,
                } = &mut state.processing_state
            {
                *wait_queue_position = TryInto::<i64>::try_into(queue_position).unwrap_or(i64::MAX);
                *update_event = Some((
                    state.content_owner.into(),
                    state.to_content_processing_state_changed(),
                ));
            }
        }

        let state_of_popped_item = {
            let state = write.processing_states.get_mut(&processing_id)?;
            state.processing_state = ContentProcessingStateInternal::Processing;
            state.clone()
        };

        drop(write);

        for (account_id, state_change) in queue_position_update_events
            .into_iter()
            .take_while(|v| v.is_some())
            .flatten()
        {
            events
                .send_content_processing_state_changed_to_client(account_id, state_change)
                .await;
        }

        events
            .send_content_processing_state_changed_to_client(
                state_of_popped_item.content_owner,
                state_of_popped_item.to_content_processing_state_changed(),
            )
            .await;

        Some(state_of_popped_item)
    }

    pub async fn get_current_state(
        &self,
        account_id: AccountIdInternal,
    ) -> Option<ContentProcessingState> {
        let key = account_id.id;
        self.data.read().await.processing_states.get(&key).map(|d| {
            d.processing_state
                .to_external(d.new_content_params.processing_id_from_client)
        })
    }
}

#[derive(Default)]
pub struct Data {
    queue: VecDeque<AccountIdDb>,
    /// One processing state is tracked per account.
    processing_states: HashMap<AccountIdDb, ProcessingState>,
}

impl Data {
    pub fn processing_states_mut(&mut self) -> &mut HashMap<AccountIdDb, ProcessingState> {
        &mut self.processing_states
    }
}

#[derive(Debug, Clone)]
pub struct ProcessingState {
    pub content_owner: AccountIdInternal,
    pub slot: ContentSlot,
    pub processing_state: ContentProcessingStateInternal,
    pub tmp_raw_img: TmpContentFile,
    pub tmp_img: TmpContentFile,
    pub new_content_params: NewContentParams,
}

impl ProcessingState {
    pub fn to_key(&self) -> AccountIdDb {
        self.content_owner.id
    }

    pub fn to_content_processing_state_changed(&self) -> ContentProcessingStateChanged {
        ContentProcessingStateChanged {
            processing_id_from_client: self.new_content_params.processing_id_from_client,
            new_state: self.processing_state,
        }
    }
}

#[derive(Debug)]
pub struct UploadInfo {
    /// Source file for content processing
    pub tmp_raw_img: TmpContentFile,
    /// Target file for content processing
    pub tmp_img: TmpContentFile,
    pub upload_permit: UploadPermit,
}
