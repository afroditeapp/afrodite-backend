use std::{fmt::Debug, path::PathBuf};

use api_client::{
    apis::media_api::{get_content_slot_state, put_moderation_request, put_pending_profile_content, put_pending_security_content_info},
    manual_additions::put_content_to_content_slot_fixed,
    models::{content_processing_state, ContentId, ContentProcessingStateType, EventToClient, EventType, MediaContentType, ModerationRequestContent, SetProfileContent},
};
use async_trait::async_trait;
use config::bot_config_file::{BotConfigFile, BotInstanceConfig, Gender};
use error_stack::{Result, ResultExt};

use super::{super::super::client::TestError, BotAction, BotState};
use crate::bot::utils::image::ImageProvider;

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
    pub random_if_not_defined_in_config: bool,
    pub copy_to_slot: Option<i32>,
    /// Add mark to the image
    pub mark_copied_image: bool,
}

impl SendImageToSlot {
    /// Slot 0 will be used as secure capture every time
    pub const fn slot(slot: i32) -> Self {
        Self {
            slot,
            random_if_not_defined_in_config: false,
            copy_to_slot: None,
            mark_copied_image: false,
        }
    }
}

fn img_for_bot(bot: &BotInstanceConfig, config: &BotConfigFile) -> std::result::Result<Option<PathBuf>, std::io::Error> {
    if let Some(image) = bot.get_img(config) {
        Ok(Some(image))
    } else {
        let dir = match bot.gender {
            Gender::Man => config.man_image_dir.clone(),
            Gender::Woman => config.woman_image_dir.clone(),
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
        let img_data = if self.random_if_not_defined_in_config {
            let img_path = if let Some(bot) = state.bot_config_file.bot.get(state.bot_id as usize) {
                img_for_bot(bot, &state.bot_config_file)
            } else if let Some(dir) = &state.bot_config_file.man_image_dir {
                ImageProvider::random_image_from_directory(&dir)
            } else {
                Ok(None)
            };

            match img_path {
                Ok(Some(img_path)) =>
                    std::fs::read(img_path)
                        .unwrap_or_else(|e| {
                            tracing::error!("{e:?}");
                            ImageProvider::random_jpeg_image()
                        }),
                Ok(None) =>
                    ImageProvider::random_jpeg_image(),
                Err(e) => {
                    tracing::error!("{e:?}");
                    ImageProvider::random_jpeg_image()
                }
            }
        } else {
            ImageProvider::random_jpeg_image()
        };

        let _ = put_content_to_content_slot_fixed(
            state.api.media(),
            self.slot,
            self.slot == 0, // secure capture
            MediaContentType::JpegImage,
            img_data.clone(),
        )
        .await
        .change_context(TestError::ApiRequest)?;

        async fn wait_for_content_id(slot: i32, state: &mut BotState) -> Result<ContentId, TestError> {
            state.wait_event(|e|
                match e.content_processing_state_changed.as_ref() {
                    Some(Some(content_processing_state)) =>
                        content_processing_state.new_state.state == ContentProcessingStateType::Completed,
                    _ => false,
                }
            ).await?;

            loop {
                let slot_state = get_content_slot_state(state.api.media(), slot)
                    .await
                    .change_context(TestError::ApiRequest)?;

                match slot_state.state {
                    ContentProcessingStateType::Empty | ContentProcessingStateType::Failed =>
                        return Err(TestError::ApiRequest.report()),
                    ContentProcessingStateType::Processing | ContentProcessingStateType::InQueue =>
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await,
                    ContentProcessingStateType::Completed =>
                        return Ok(*slot_state
                            .content_id
                            .flatten()
                            .expect("Content ID is missing")),
                }
            }
        }

        let content_id = wait_for_content_id(self.slot, state).await?;
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
            let _ = put_content_to_content_slot_fixed(
                state.api.media(),
                slot,
                if slot == 0 { true } else { false }, // secure capture
                MediaContentType::JpegImage,
                img_data,
            )
            .await
            .change_context(TestError::ApiRequest)?;

            let content_id = wait_for_content_id(slot, state).await?;
            state.media.slots[slot as usize] = Some(content_id);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct MakeModerationRequest {
    /// Use the first slot as secure capture slot
    pub slot_0_secure_capture: bool,
}

#[async_trait]
impl BotAction for MakeModerationRequest {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let mut content_ids: Vec<Option<Box<ContentId>>> = vec![];

        if self.slot_0_secure_capture {
            content_ids.push(
                Box::new(state.media.slots[0].clone().unwrap_or(ContentId {
                    content_id: uuid::Uuid::new_v4(),
                }))
                .into(),
            );
        }

        content_ids.push(
            state.media.slots[1]
                .clone()
                .map(|id| Box::new(id))
                .unwrap_or(Box::new(ContentId {
                    content_id: uuid::Uuid::new_v4(),
                }))
                .into(),
        );

        content_ids.push(state.media.slots[2].clone().map(|id| Box::new(id)));

        let new = ModerationRequestContent {
            content0: content_ids[0].clone().expect("Content ID is missing"),
            content1: content_ids.get(1).cloned(),
            content2: content_ids.get(2).cloned(),
            content3: None,
            content4: None,
            content5: None,
            content6: None,
        };

        put_moderation_request(state.api.media(), new)
            .await
            .change_context(TestError::ApiRequest)?;
        Ok(())
    }
}


#[derive(Debug)]
pub struct SetPendingContent {
    pub security_content_slot_i: Option<usize>,
    pub content_0_slot_i: Option<usize>,
}

#[async_trait]
impl BotAction for SetPendingContent {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        if let Some(i) = self.security_content_slot_i {
            let content_id = state.media.slots[i].clone().unwrap();
            put_pending_security_content_info(state.api.media(), content_id)
                .await
                .change_context(TestError::ApiRequest)?;
        }

        if let Some(i) = self.content_0_slot_i {
            let content_id = state.media.slots[i].clone().unwrap();
            let info = SetProfileContent {
                content_id_0: content_id.into(),
                ..SetProfileContent::default()
            };
            put_pending_profile_content(state.api.media(), info)
                .await
                .change_context(TestError::ApiRequest)?;
        }

        Ok(())
    }
}
