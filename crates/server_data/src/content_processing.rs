use std::collections::{HashMap, VecDeque};

use model::{
    AccountIdDb, AccountIdInternal, ContentProcessingId, ContentProcessingState,
    ContentProcessingStateChanged, ContentSlot, NewContentParams,
};
use server_common::result::WrappedResultExt;
use tokio::sync::{mpsc::{self, UnboundedReceiver, UnboundedSender}, RwLock};
use tracing::warn;

use crate::{event::EventManagerWithCacheReference, file::utils::TmpContentFile};

use crate::result::Result;

#[derive(thiserror::Error, Debug)]
pub enum ContentProcessingError {
    #[error("Event sending failed")]
    EventSendingFailed,
}

#[derive(Debug)]
pub struct ContentProcessingReceiver(pub UnboundedReceiver<ProcessingKey>);

pub struct ContentProcessingManagerData {
    event_queue: UnboundedSender<ProcessingKey>,
    data: RwLock<Data>,
}

impl ContentProcessingManagerData {
    pub fn new() -> (Self, ContentProcessingReceiver) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let notifier = ContentProcessingReceiver(receiver);
        let data = Self {
            event_queue: sender,
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
    ) -> Result<(), ContentProcessingError> {
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
            in_event_queue: true,
        };
        let was_already_in_event_queue = processing_states.insert(key, state)
            .map(|v| v.in_event_queue)
            .unwrap_or_default();

        if !was_already_in_event_queue {
            self.event_queue.send(key)
                .change_context(ContentProcessingError::EventSendingFailed)?
        }

        drop(write);

        Ok(())
    }

    pub async fn pop_from_queue(
        &self,
        events: EventManagerWithCacheReference<'_>,
        processing_id: ProcessingKey,
    ) -> Option<ProcessingState> {
        let mut write = self.data.write().await;
        let (queue, processing_states) = write.split();
        let processing_id_index = queue.iter().enumerate().find(|v| v.1 == &processing_id)?.0;
        queue.remove(processing_id_index);

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
        state.in_event_queue = false;
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

#[derive(Debug, Default)]
pub struct Data {
    queue: VecDeque<ProcessingKey>,
    /// Nothing is removed from here as content slots limit
    /// memory usage enough.
    processing_states: HashMap<ProcessingKey, ProcessingState>,
}

impl Data {
    pub fn new() -> Self {
        Self::default()
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
    pub in_event_queue: bool,
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
