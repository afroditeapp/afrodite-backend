use std::{fmt::Debug, path::PathBuf};

use api_client::{
    apis::media_api::{
        get_content_processing_state, put_profile_content, put_security_content_info,
        put_upload_content,
    },
    models::{ContentId, ContentProcessingStateType, MediaContentUploadType, SetProfileContent},
};
use async_trait::async_trait;
use config::bot_config_file::{BaseBotConfig, BotConfigFile, Gender};
use error_stack::{Result, ResultExt};
use test_mode_utils::client::TestError;

use super::{BotAction, BotState};
use crate::utils::image::ImageProvider;

#[derive(Debug, Default)]
pub struct MediaState {
    /// Max slot count and one extra to allow current
    /// content sending code work when testing that sending
    /// content to the extra slot will make an error.
    slots: [Option<ContentId>; 8],
}

impl MediaState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub struct SendImageToSlot {
    pub slot: i32,
    pub copy_to_slot: Option<i32>,
    /// Add mark to the image
    pub mark_copied_image: bool,
}

impl SendImageToSlot {
    /// Slot 0 will be used as secure capture every time
    pub const fn slot(slot: i32) -> Self {
        Self {
            slot,
            copy_to_slot: None,
            mark_copied_image: false,
        }
    }

    async fn send_to_slot(&self, state: &mut BotState) -> Result<(), TestError> {
        let img_data = if state.get_bot_config().random_color_image() {
            ImageProvider::random_jpeg_image()
        } else {
            let img_path = img_for_bot(state.get_bot_config(), &state.bot_config_file);
            match img_path {
                Ok(Some(img_path)) => std::fs::read(img_path).unwrap_or_else(|e| {
                    tracing::error!("{e:?}");
                    ImageProvider::default_jpeg_image()
                }),
                Ok(None) => ImageProvider::default_jpeg_image(),
                Err(e) => {
                    tracing::error!("{e:?}");
                    ImageProvider::default_jpeg_image()
                }
            }
        };

        let processing_id_from_client = i32::from(state.next_ws_request_id());
        let upload_result = put_upload_content(
            &state.api(),
            self.slot,
            processing_id_from_client,
            self.slot == 0, // secure capture
            MediaContentUploadType::Image,
            img_data.clone(),
        )
        .await
        .change_context(TestError::ApiRequest)?;

        if upload_result.error.unwrap_or(false) {
            return Err(TestError::ApiRequest.report());
        }

        async fn wait_for_content_id(
            processing_id_from_client: u8,
            state: &mut BotState,
        ) -> Result<ContentId, TestError> {
            let event_waiting_result = state
                .wait_event(|e| match e.content_processing_state_changed.as_ref() {
                    Some(content_processing_state) => {
                        content_processing_state.processing_id_from_client
                            == processing_id_from_client
                            && content_processing_state.new_state.state
                                == Some(Some(ContentProcessingStateType::Completed))
                    }
                    _ => false,
                })
                .await;

            match event_waiting_result {
                Ok(()) => (),
                Err(e) if e.current_context() == &TestError::EventReceivingTimeout => (),
                Err(e) => return Err(e),
            }

            loop {
                let state_from_api = get_content_processing_state(&state.api())
                    .await
                    .change_context(TestError::ApiRequest)?;

                match state_from_api.processing_id_from_client.flatten() {
                    None => return Err(TestError::ApiRequest.report()),
                    Some(id) if id != i32::from(processing_id_from_client) => {
                        return Err(TestError::ApiRequest.report());
                    }
                    Some(_) => (),
                }

                match state_from_api.state.flatten() {
                    None
                    | Some(ContentProcessingStateType::Failed)
                    | Some(ContentProcessingStateType::NsfwDetected) => {
                        return Err(TestError::ApiRequest.report());
                    }
                    Some(ContentProcessingStateType::Processing)
                    | Some(ContentProcessingStateType::InQueue) => {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await
                    }
                    Some(ContentProcessingStateType::Completed) => {
                        match state_from_api.cid.clone().flatten() {
                            None => return Err(TestError::ApiRequest.report()),
                            Some(cid) => return Ok(*cid),
                        }
                    }
                }
            }
        }

        let content_id = wait_for_content_id(processing_id_from_client as u8, state).await?;
        state.media.slots[self.slot as usize] = Some(content_id);

        let img_data = if self.mark_copied_image {
            ImageProvider::mark_jpeg_image(&img_data).unwrap_or_else(|e| {
                tracing::error!("{e:?}");
                img_data
            })
        } else {
            img_data
        };

        if let Some(slot) = self.copy_to_slot {
            let processing_id_from_client = i32::from(state.next_ws_request_id());
            let upload_result = put_upload_content(
                &state.api(),
                slot,
                processing_id_from_client,
                slot == 0, // slot 0 is for secure capture
                MediaContentUploadType::Image,
                img_data,
            )
            .await
            .change_context(TestError::ApiRequest)?;

            if upload_result.error.unwrap_or(false) {
                return Err(TestError::ApiRequest.report());
            }

            let content_id = wait_for_content_id(processing_id_from_client as u8, state).await?;
            state.media.slots[slot as usize] = Some(content_id);
        }

        Ok(())
    }
}

fn img_for_bot(
    bot: &BaseBotConfig,
    config: &BotConfigFile,
) -> std::result::Result<Option<PathBuf>, std::io::Error> {
    if let Some(image) = bot.get_img(config) {
        Ok(Some(image))
    } else {
        let dir = match bot.img_dir_gender() {
            Gender::Man => config.image_dir.man.clone(),
            Gender::Woman => config.image_dir.woman.clone(),
        };
        if let Some(dir) = dir {
            ImageProvider::random_image_from_directory(&dir)
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl BotAction for SendImageToSlot {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let events_enabled = state.are_events_enabled();
        if !events_enabled {
            state.enable_events();
        }
        let result = self.send_to_slot(state).await;
        if !events_enabled {
            state.disable_events();
        }
        result
    }
}

#[derive(Debug)]
pub struct SetContent {
    pub security_content_slot_i: Option<usize>,
    pub content_0_slot_i: Option<usize>,
}

#[async_trait]
impl BotAction for SetContent {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        if let Some(i) = self.security_content_slot_i {
            let content_id = state.media.slots[i].clone().unwrap();
            put_security_content_info(&state.api(), content_id)
                .await
                .change_context(TestError::ApiRequest)?;
        }

        if let Some(i) = self.content_0_slot_i {
            let content_id = state.media.slots[i].clone().unwrap();
            let bot_info = state.get_bot_config();

            let info = SetProfileContent {
                content: vec![content_id],
                grid_crop_size: bot_info.grid_crop_size,
                grid_crop_x: bot_info.grid_crop_x,
                grid_crop_y: bot_info.grid_crop_y,
            };
            put_profile_content(&state.api(), info)
                .await
                .change_context(TestError::ApiRequest)?;
        }

        Ok(())
    }
}
