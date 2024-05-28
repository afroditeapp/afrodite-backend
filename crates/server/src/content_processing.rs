use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use model::{
    AccountIdDb, AccountIdInternal, ContentProcessingId, ContentProcessingState,
    ContentProcessingStateChanged, ContentSlot, MediaContentType, NewContentParams,
};
use server_data::content_processing::{notify_client, ContentProcessingNotify, ProcessingState};
use simple_backend::{app::SimpleBackendAppState, image::ImageProcess, ServerQuitWatcher};
use simple_backend_config::args::InputFileType;
use tokio::{
    sync::{Notify, RwLock},
    task::JoinHandle,
};
use tracing::{error, warn};

use crate::{
    app::{AppState, ContentProcessingProvider, EventManagerProvider, WriteData},
};
use server_data::file::utils::TmpContentFile;
use server_data::event::EventManagerWithCacheReference;
use server_common::result::{Result, WrappedResultExt};

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

pub struct ContentProcessingManager {
    state: AppState,
}

impl ContentProcessingManager {
    pub fn new_manager(
        notifier: ContentProcessingNotify,
        state: AppState,
        quit_notification: ServerQuitWatcher,
    ) -> ContentProcessingManagerQuitHandle {
        let manager = Self { state };

        let task = tokio::spawn(manager.run(notifier, quit_notification));

        ContentProcessingManagerQuitHandle { task }
    }

    pub async fn run(
        self,
        notifier: ContentProcessingNotify,
        mut quit_notification: ServerQuitWatcher,
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
            MediaContentType::JpegImage => ImageProcess::start_image_process(
                content.tmp_raw_img.as_path(),
                InputFileType::JpegImage,
                content.tmp_img.as_path(),
            )
            .await
            .change_context(ContentProcessingError::ContentProcessingFailed),
        };

        let mut write = self.state.content_processing().data().write().await;
        if let Some(state) = write.processing_states_mut().get_mut(&content.to_key()) {
            let result = self.if_successful_save_to_database(result, state).await;
            match result {
                Ok(()) => {
                    state
                        .processing_state
                        .change_to_completed(state.processing_id.to_content_id());
                }
                Err(e) => {
                    state.processing_state.change_to_failed();
                    error!("Content processing error: {:?}", e);
                }
            }

            notify_client(&self.state.event_manager(), state).await;
        } else {
            warn!("Content processing state not found");
            match result {
                Ok(()) => (),
                Err(e) => {
                    error!("Content processing error: {:?}", e);
                }
            }
        }

        if let Err(e) = content.tmp_raw_img.remove_if_exists().await {
            warn!("content.tmp_raw_img removing failed {:?}", e)
        }
    }

    pub async fn if_successful_save_to_database(
        &self,
        result: Result<(), ContentProcessingError>,
        state: &mut ProcessingState,
    ) -> Result<(), ContentProcessingError> {
        let () = result?;

        let state_copy = state.clone();
        self.state
            .write(move |cmds| async move {
                cmds.media()
                    .save_to_slot(
                        state_copy.content_owner,
                        state_copy.processing_id.to_content_id(),
                        state_copy.slot,
                        state_copy.new_content_params,
                    )
                    .await
            })
            .await
            .change_context(ContentProcessingError::DatabaseError)
    }
}
