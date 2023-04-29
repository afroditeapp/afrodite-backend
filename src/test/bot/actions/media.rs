
use std::fmt::{Debug};

use api_client::{apis::{media_api::{put_moderation_request}}, models::{ContentId, ModerationRequestContent}, manual_additions::put_image_to_moderation_slot_fixed};
use async_trait::async_trait;

use error_stack::{Result};



use super::{super::super::client::{TestError}, BotAction};

use crate::{
    utils::IntoReportExt, test::bot::utils::image::ImageProvider,
};

use super::BotState;


#[derive(Debug, Default)]
pub struct MediaState {
    slots: [Option<ContentId>; 3],
}

impl MediaState {
    pub fn new() -> Self {
        Self::default()
    }
}


#[derive(Debug)]
pub struct SendImageToSlot(pub i32);

#[async_trait]
impl BotAction for SendImageToSlot {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let content_id = put_image_to_moderation_slot_fixed(
            state.api.media(), self.0, ImageProvider::jpeg_image()
        ).await.into_error(TestError::ApiRequest)?;
        state.media.slots[self.0 as usize] = Some(content_id);
        Ok(())
    }
}

#[derive(Debug)]
pub struct MakeModerationRequest {
    pub camera: bool,
}

#[async_trait]
impl BotAction for MakeModerationRequest {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let new = ModerationRequestContent {
            camera_image: self.camera,
            image1: Box::new(
                state.media.slots[0].clone().unwrap_or(ContentId { content_id: uuid::Uuid::new_v4() })
            ),
            image2: state.media.slots[1].clone().map(|id| Box::new(id)),
            image3: state.media.slots[2].clone().map(|id| Box::new(id)),
        };

        put_moderation_request(state.api.media(), new)
            .await.into_error(TestError::ApiRequest)?;
        Ok(())
    }
}
