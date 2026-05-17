use config::Config;
use model::{
    AdminBotNotificationTypes, AdminNotificationTypes, ContentId, ContentProcessingStateChanged,
    ContentProcessingStateInternal,
};
use model_media::MediaContentUploadType;
use server_api::{
    app::{ContentProcessingProvider, EventManagerProvider, WriteData},
    db_write_raw,
};
use server_common::result::{Result, WrappedResultExt};
use server_data::{
    app::{GetConfig, ReadData},
    content_processing::{ContentProcessingReceiver, ProcessingState},
    read::GetReadCommandsCommon,
};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use server_state::{S, app::AdminNotificationProvider};
use simple_backend::{
    ServerQuitWatcher,
    image::{ImageProcess, ImageProcessError},
};
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

    async fn run(
        self,
        receiver: ContentProcessingReceiver,
        mut quit_notification: ServerQuitWatcher,
    ) {
        loop {
            tokio::select! {
                _ = receiver.0.notified() => {
                    while let Some(content) = self
                        .state
                        .content_processing()
                        .pop_from_queue(self.state.event_manager())
                        .await
                    {
                        let content_owner = content.content_owner;
                        let new_state = self.handle_content(content).await;
                        self.state
                            .content_processing()
                            .set_processing_phase_idle(content_owner)
                            .await;
                        if let Some(new_state) = new_state {
                            self.state
                                .event_manager()
                                .send_content_processing_state_changed_to_client(
                                    content_owner,
                                    new_state,
                                )
                                .await;
                        }
                    }
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }

    async fn handle_content(
        &self,
        content: ProcessingState,
    ) -> Option<ContentProcessingStateChanged> {
        let result = match content.new_content_params.content_type {
            MediaContentUploadType::Image => {
                let state = self.state.clone();
                ImageProcess::start_image_process(
                    self.state.config().simple_backend(),
                    || async move {
                        let dynamic_config = state
                            .read()
                            .media_admin()
                            .image_processing_config()
                            .await
                            .change_context(ImageProcessError::ConfigLoading)
                            .map_err(|e| e.into_report())?
                            .unwrap_or_default();
                        Ok(dynamic_config)
                    },
                    content.tmp_raw_img.as_path(),
                    content.tmp_img.as_path(),
                )
                .await
                .change_context(ContentProcessingError::ContentProcessingFailed)
            }
        };

        let mut write = self.state.content_processing().data().write().await;
        let new_state = if let Some(state) =
            write.processing_states_mut().get_mut(&content.to_key())
        {
            let result = self
                .if_successful_and_no_nsfw_then_save_to_database(self.state.config(), result, state)
                .await;
            match result {
                Ok(ImgInfo::ProcessedSuccessfully {
                    face_detected,
                    content_id,
                }) => {
                    state.processing_state = ContentProcessingStateInternal::Completed {
                        content_id,
                        face_detected,
                    };
                }
                Ok(ImgInfo::NsfwDetected) => {
                    state.processing_state = ContentProcessingStateInternal::NsfwDetected;
                }
                Err(e) => {
                    state.processing_state = ContentProcessingStateInternal::Failed;
                    error!("Content processing error: {:?}", e);
                }
            }

            Some(state.to_content_processing_state_changed())
        } else {
            warn!("Content processing state not found");
            match result {
                Ok(_) => None,
                Err(e) => {
                    error!("Content processing error: {:?}", e);
                    None
                }
            }
        };

        if let Err(e) = content.tmp_raw_img.overwrite_and_remove_if_exists().await {
            warn!("content.tmp_raw_img removing failed {:?}", e)
        }

        new_state
    }

    async fn if_successful_and_no_nsfw_then_save_to_database(
        &self,
        config: &Config,
        result: Result<ImageProcessingInfo, ContentProcessingError>,
        state: &mut ProcessingState,
    ) -> Result<ImgInfo, ContentProcessingError> {
        let info = result?;
        if info.nsfw_detected {
            return Ok(ImgInfo::NsfwDetected);
        }
        let face_detected =
            if let Some(face_detected) = config.simple_backend().debug_face_detection_result() {
                face_detected
            } else {
                info.face_detected
            };

        // Check if uploader is a bot
        let is_bot = self
            .state
            .read()
            .common()
            .is_bot(state.content_owner)
            .await
            .change_context(ContentProcessingError::DatabaseError)?;

        let state_copy = state.clone();
        let content_id = db_write_raw!(self.state, move |cmds| {
            cmds.media()
                .save_img(
                    state_copy.content_owner,
                    state_copy.tmp_img,
                    state_copy.slot,
                    state_copy.new_content_params,
                    face_detected,
                    if is_bot { Some(true) } else { None },
                )
                .await
        })
        .await
        .change_context(ContentProcessingError::DatabaseError)?;

        self.state
            .admin_notification()
            .send_notification_if_needed(AdminNotificationTypes::ModerateInitialMediaContentBot)
            .await;
        self.state
            .admin_notification()
            .send_notification_if_needed(AdminNotificationTypes::ModerateMediaContentBot)
            .await;
        self.state
            .admin_notification()
            .send_bot_notification_if_needed(
                AdminBotNotificationTypes::VERIFY_MEDIA_CONTENT_FACE_BOT,
            )
            .await;

        Ok(ImgInfo::ProcessedSuccessfully {
            face_detected,
            content_id,
        })
    }
}

enum ImgInfo {
    ProcessedSuccessfully {
        face_detected: bool,
        content_id: ContentId,
    },
    NsfwDetected,
}
