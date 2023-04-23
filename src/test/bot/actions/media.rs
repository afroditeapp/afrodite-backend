
use std::fmt::{Debug, Display};

use api_client::{apis::{account_api::{post_register, post_login}, profile_api::{post_profile, get_profile, get_default_profile}, media_api::{put_image_to_moderation_slot, put_moderation_request}}, models::{Profile, ContentId, NewModerationRequest}, manual_additions::put_image_to_moderation_slot_fixed};
use async_trait::async_trait;
use nalgebra::U8;

use error_stack::{Result, FutureExt, ResultExt, Report};

use tracing::{error, log::warn};

use super::{super::super::client::{ApiClient, TestError}, BotAction};

use crate::{
    api::model::AccountId,
    config::args::{Test, TestMode},
    utils::IntoReportExt, server::database::file::file::ImageSlot, test::bot::utils::image::ImageProvider,
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
        let new = NewModerationRequest {
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
