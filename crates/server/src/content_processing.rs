use config::Config;
use model::ContentId;
use model_media::MediaContentType;
use server_api::{
    app::{ContentProcessingProvider, EventManagerProvider, WriteData},
    db_write_raw,
};
use server_common::result::{Result, WrappedResultExt};
use server_data::{
    app::GetConfig,
    content_processing::{notify_client, ContentProcessingReceiver, ProcessingState},
};
use server_data_media::write::GetWriteCommandsMedia;
use server_state::S;
use simple_backend::{image::ImageProcess, ServerQuitWatcher};
use simple_backend_config::args::InputFileType;
use simple_backend_image_process::ImageProcessingInfo;
use tokio::task::JoinHandle;
use tracing::{error, warn};

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
    state: S,
}

impl ContentProcessingManager {
    pub fn new_manager(
        receiver: ContentProcessingReceiver,
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> ContentProcessingManagerQuitHandle {
        let manager = Self { state };

        let task = tokio::spawn(manager.run(receiver, quit_notification));

        ContentProcessingManagerQuitHandle { task }
    }

    pub async fn run(
        self,
        mut receiver: ContentProcessingReceiver,
        mut quit_notification: ServerQuitWatcher,
    ) {
        loop {
            tokio::select! {
                item = receiver.0.recv() => {
                    match item {
                        Some(item) => {
                            let new_content = self.state.content_processing().pop_from_queue(self.state.event_manager(), item).await;
                            if let Some(content) = new_content {
                                self.handle_content(content).await;
                            }
                        }
                        None => {
                            error!("Content processing event channel is broken");
                            return;
                        },
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
            let result = self
                .if_successful_save_to_database(self.state.config(), result, state)
                .await;
            match result {
                Ok(ImgInfo { face_detected, content_id }) => {
                    state
                        .processing_state
                        .change_to_completed(content_id, face_detected);
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
                Ok(_) => (),
                Err(e) => {
                    error!("Content processing error: {:?}", e);
                }
            }
        }

        if let Err(e) = content.tmp_raw_img.overwrite_and_remove_if_exists().await {
            warn!("content.tmp_raw_img removing failed {:?}", e)
        }
    }

    async fn if_successful_save_to_database(
        &self,
        config: &Config,
        result: Result<ImageProcessingInfo, ContentProcessingError>,
        state: &mut ProcessingState,
    ) -> Result<ImgInfo, ContentProcessingError> {
        let info = result?;
        let face_detected =
            if let Some(face_detected) = config.simple_backend().debug_face_detection_result() {
                face_detected
            } else {
                info.face_detected
            };

        let state_copy = state.clone();
        let content_id = db_write_raw!(self.state, move |cmds| {
            cmds.media()
                .save_img(
                    state_copy.content_owner,
                    state_copy.tmp_img,
                    state_copy.slot,
                    state_copy.new_content_params,
                    face_detected,
                )
                .await
        })
        .await
        .change_context(ContentProcessingError::DatabaseError)?;

        Ok(ImgInfo {
            face_detected,
            content_id,
        })
    }
}

struct ImgInfo {
    face_detected: bool,
    content_id: ContentId,
}
