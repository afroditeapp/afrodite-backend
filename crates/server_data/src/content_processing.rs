use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use model::{
    AccountIdDb, AccountIdInternal, ContentProcessingId, ContentProcessingState,
    ContentProcessingStateChanged, ContentSlot, NewContentParams,
};
use tokio::sync::{Notify, RwLock};
use tracing::warn;

use crate::{event::EventManagerWithCacheReference, file::utils::TmpContentFile};

#[derive(Debug, Clone)]
pub struct ContentProcessingNotify(pub Arc<Notify>);

pub struct ContentProcessingManagerData {
    new_processing_request: ContentProcessingNotify,
    data: RwLock<Data>,
}

impl ContentProcessingManagerData {
    pub fn new() -> (Self, ContentProcessingNotify) {
        let notifier = ContentProcessingNotify(Arc::new(Notify::new()));
        let data = Self {
            new_processing_request: notifier.clone(),
            data: RwLock::new(Data::new()),
        };
        (data, notifier)
    }

    pub fn data(&self) -> &RwLock<Data> {
        &self.data
    }

    /// Queue new content. Old one will be replaced.
    pub async fn queue_new_content(
        &self,
        content_owner: AccountIdInternal,
        slot: ContentSlot,
        content_info: NewContentInfo,
        new_content_params: NewContentParams,
    ) {
        let mut write = self.data.write().await;
        let (queue, processing_states) = write.split();
        let processing_id = content_info.processing_id;

        // Reuse the same queue position. This might happen if API is used wrongly.
        let key = ProcessingKey::new(content_owner, slot);
        let queue_position = match queue.iter().enumerate().find(|(_, k)| **k == key) {
            Some((old_i, _)) => old_i as u64 + 1,
            None => {
                queue.push_back(key);
                queue.len() as u64
            }
        };

        let state = ProcessingState {
            processing_id,
            content_owner,
            slot,
            processing_state: ContentProcessingState::in_queue_state(queue_position),
            tmp_img: content_info.tmp_img,
            tmp_raw_img: content_info.tmp_raw_img,
            new_content_params,
        };
        processing_states.insert(key, state);
        drop(write);
        self.new_processing_request.0.notify_one();
    }

    pub async fn pop_from_queue(
        &self,
        events: EventManagerWithCacheReference<'_>,
    ) -> Option<ProcessingState> {
        let mut write = self.data.write().await;
        let (queue, processing_states) = write.split();
        let processing_id = queue.pop_front()?;

        // Update queue position numbers
        for (index, processing_id_in_queue) in queue.iter_mut().enumerate() {
            if let Some(state) = processing_states.get_mut(processing_id_in_queue) {
                if let Some(number) = state.processing_state.wait_queue_position.as_mut() {
                    *number = index as u64 + 1;
                    notify_client(&events, state).await;
                }
            }
        }

        let state = processing_states.get_mut(&processing_id)?;
        state.processing_state.change_to_processing();
        notify_client(&events, state).await;

        Some(state.clone())
    }

    pub async fn get_state(
        &self,
        account_id: AccountIdInternal,
        slot: ContentSlot,
    ) -> Option<ContentProcessingState> {
        let key = ProcessingKey::new(account_id, slot);
        self.data
            .read()
            .await
            .processing_states
            .get(&key)
            .map(|d| d.processing_state.clone())
    }
}

pub struct Data {
    queue: VecDeque<ProcessingKey>,
    processing_states: HashMap<ProcessingKey, ProcessingState>,
}

impl Data {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            processing_states: HashMap::new(),
        }
    }

    pub fn split(
        &mut self,
    ) -> (
        &mut VecDeque<ProcessingKey>,
        &mut HashMap<ProcessingKey, ProcessingState>,
    ) {
        (&mut self.queue, &mut self.processing_states)
    }

    pub fn processing_states(&self) -> &HashMap<ProcessingKey, ProcessingState> {
        &self.processing_states
    }

    pub fn processing_states_mut(&mut self) -> &mut HashMap<ProcessingKey, ProcessingState> {
        &mut self.processing_states
    }
}

#[derive(Debug, Clone)]
pub struct ProcessingState {
    pub processing_id: ContentProcessingId,
    pub content_owner: AccountIdInternal,
    pub slot: ContentSlot,
    pub processing_state: ContentProcessingState,
    pub tmp_raw_img: TmpContentFile,
    pub tmp_img: TmpContentFile,
    pub new_content_params: NewContentParams,
}

impl ProcessingState {
    pub fn to_key(&self) -> ProcessingKey {
        ProcessingKey::new(self.content_owner, self.slot)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessingKey {
    content_owner: AccountIdDb,
    slot: ContentSlot,
}

impl ProcessingKey {
    fn new(account_id: AccountIdInternal, slot: ContentSlot) -> Self {
        Self {
            content_owner: account_id.id,
            slot,
        }
    }
}

// TODO: add extension method to EventManager?

pub async fn notify_client(
    event_manager: &EventManagerWithCacheReference<'_>,
    state: &ProcessingState,
) {
    let state_change = ContentProcessingStateChanged {
        id: state.processing_id,
        new_state: state.processing_state.clone(),
    };

    if let Err(e) = event_manager
        .send_connected_event(
            state.content_owner,
            model::EventToClientInternal::ContentProcessingStateChanged(state_change),
        )
        .await
    {
        warn!("Event sending failed {}", e);
    }
}

#[derive(Debug, Clone)]
pub struct NewContentInfo {
    pub processing_id: ContentProcessingId,
    pub tmp_raw_img: TmpContentFile,
    pub tmp_img: TmpContentFile,
}
