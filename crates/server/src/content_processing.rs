use std::{collections::{HashMap, VecDeque}, sync::Arc};


use model::{ContentProcessingId, ContentProcessingState, ContentSlot, AccountIdInternal, ContentProcessingStateChanged, NewContentParams, AccountIdDb, MediaContentType};
use simple_backend::{ServerQuitWatcher, app::SimpleBackendAppState, image::{ImageProcess}};

use tokio::{task::JoinHandle, sync::{RwLock, Notify}};
use tracing::{warn, error};


use crate::{event::EventManager, app::{AppState, ContentProcessingProvider, EventManagerProvider, WriteData}, data::file::utils::TmpContentFile};

use error_stack::{Result, FutureExt, ResultExt};

#[derive(thiserror::Error, Debug)]
pub enum ContentProcessingError {
    #[error("Content processing error")]
    ContentProcessingFailed,

    #[error("Database update error")]
    DatabaseError,
}

#[derive(Debug)]
pub struct ContentProcessingManagerQuitHandle {
    task: JoinHandle<()>,
}

impl ContentProcessingManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("ContentProcessingManager quit failed. Error: {:?}", e);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewContentInfo {
    pub processing_id: ContentProcessingId,
    pub tmp_raw_img: TmpContentFile,
    pub tmp_img: TmpContentFile,
}

#[derive(Debug, Clone)]
pub struct ProcessingState {
    processing_id: ContentProcessingId,
    content_owner: AccountIdInternal,
    slot: ContentSlot,
    processing_state: ContentProcessingState,
    tmp_raw_img: TmpContentFile,
    tmp_img: TmpContentFile,
    new_content_params: NewContentParams,
}

impl ProcessingState {
    fn to_key(&self) -> ProcessingKey {
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

struct Data {
    queue: VecDeque<ProcessingKey>,
    processing_states: HashMap<ProcessingKey, ProcessingState>
}

impl Data {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            processing_states: HashMap::new(),
        }
    }

    pub fn split(&mut self) -> (&mut VecDeque<ProcessingKey>, &mut HashMap<ProcessingKey, ProcessingState>) {
        (&mut self.queue, &mut self.processing_states)
    }
}

#[derive(Debug, Clone)]
pub struct ContentProcessingNotify(Arc<Notify>);

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
            Some((old_i, _)) => {
                old_i as u64 + 1
            }
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

    pub async fn pop_from_queue(&self, events: &EventManager) -> Option<ProcessingState> {
        let mut write = self.data.write().await;
        let (queue, processing_states) = write.split();
        let processing_id = queue.pop_front()?;

        // Update queue position numbers
        for (index, processing_id_in_queue) in queue.iter_mut().enumerate() {
            if let Some(state) = processing_states.get_mut(&processing_id_in_queue) {
                if let Some(number) = state.processing_state.wait_queue_position.as_mut() {
                    *number = index as u64 + 1;
                    notify_client(events, state).await;
                }
            }
        }

        let state = processing_states.get_mut(&processing_id)?;
        state.processing_state.change_to_processing();
        notify_client(events, state).await;

        Some(state.clone())
    }

    pub async fn get_state(&self, account_id: AccountIdInternal, slot: ContentSlot) -> Option<ContentProcessingState> {
        let key = ProcessingKey::new(account_id, slot);
        self.data.read().await.processing_states.get(&key).map(|d| d.processing_state.clone())
    }
}

pub struct ContentProcessingManager {
    state: SimpleBackendAppState<AppState>,
}

impl ContentProcessingManager {
    pub fn new(
        notifier: ContentProcessingNotify,
        state: SimpleBackendAppState<AppState>,
        quit_notification: ServerQuitWatcher,
    ) -> ContentProcessingManagerQuitHandle {
        let manager = Self {
            state,
        };

        let task = tokio::spawn(manager.run(notifier, quit_notification));

        let quit_handle = ContentProcessingManagerQuitHandle { task };

        quit_handle
    }

    pub async fn run(
        self,
        notifier: ContentProcessingNotify,
        mut quit_notification: ServerQuitWatcher
    ) {
        loop {
            tokio::select! {
                _ = notifier.0.notified() => {
                    let new_content = self.state.content_processing().pop_from_queue(self.state.event_manager()).await;
                    if let Some(content) = new_content {
                        self.handle_content(content).await;
                    }
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }

    pub async fn handle_content(&self, content: ProcessingState) {
        let result = match content.new_content_params.content_type {
            MediaContentType::JpegImage => {
                ImageProcess::start_image_process(content.tmp_raw_img.as_path(), content.tmp_img.as_path())
                    .change_context(ContentProcessingError::ContentProcessingFailed)
                    .await
            }
        };

        let mut write = self.state.content_processing().data.write().await;
        if let Some(state) = write.processing_states.get_mut(&content.to_key()) {
            let result = self.if_successful_save_to_database(result, state).await;
            match result {
                Ok(()) => {
                    state.processing_state.change_to_completed(state.processing_id.to_content_id());
                },
                Err(e) => {
                    state.processing_state.change_to_failed();
                    error!("Content processing error: {}", e);
                }
            }

            notify_client(self.state.event_manager(), state).await;
        } else {
            warn!("Content processing state not found");
            match result {
                Ok(()) => (),
                Err(e) => {
                    error!("Content processing error: {}", e);
                }
            }
        }

        if let Err(e) = content.tmp_raw_img
            .remove_if_exists()
            .await {
                warn!("content.tmp_raw_img removing failed {}", e)
            }
    }

    pub async fn if_successful_save_to_database(
        &self,
        result: Result<(), ContentProcessingError>,
        state: &mut ProcessingState
    ) -> Result<(), ContentProcessingError> {
        if let Err(e) = result {
            return Err(e);
        }

        let state_copy = state.clone();
        self.state
            .write(move |cmds| async move {
                cmds.media()
                    .save_to_slot(
                        state_copy.content_owner,
                        state_copy.processing_id.to_content_id(),
                        state_copy.slot,
                        state_copy.new_content_params
                    )
                    .await
            })
            .await
            .change_context(ContentProcessingError::DatabaseError)
    }
}

// TODO: add extension method to EventManager?

async fn notify_client(event_manager: &EventManager, state: &ProcessingState) {
    let state_change = ContentProcessingStateChanged {
        id: state.processing_id,
        new_state: state.processing_state.clone(),
    };

    if let Err(e) = event_manager.send_connected_event(
        state.content_owner,
        model::EventToClientInternal::ContentProcessingStateChanged(state_change)
    ).await {
        warn!("Event sending failed {}", e);
    }
}
