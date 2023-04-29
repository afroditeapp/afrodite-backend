
use std::fmt::{Debug, Display};

use api_client::{apis::{account_api::{post_register, post_login}, profile_api::{post_profile, get_profile, get_default_profile}, media_api}, models::Profile};
use async_trait::async_trait;
use nalgebra::U8;

use error_stack::{Result, FutureExt, ResultExt};

use tracing::{error, log::warn};

use super::{super::super::client::{ApiClient, TestError}, BotAction};

use crate::{
    config::args::{Test, TestMode},
    utils::IntoReportExt,
};

use super::BotState;




#[derive(Debug)]
pub struct ModerateMediaModerationRequest;

#[async_trait]
impl BotAction for ModerateMediaModerationRequest {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let list = media_api::patch_moderation_request_list(state.api.media())
            .await
            .into_error(TestError::ApiRequest)?;

        for request in list.list {
            let images = [Some(request.content.image1), request.content.image2, request.content.image3];
            for content_id in images.iter().flatten() {
                api_client::manual_additions::get_image_fixed(state.api.media(), &request.request_creator_id.to_string(), &content_id.to_string())
                    .await
                    .into_error(TestError::ApiRequest)?;
            }
            media_api::post_handle_moderation_request(
                state.api.media(),
                &request.request_creator_id.to_string(),
                api_client::models::HandleModerationRequest { accept: true }
            )
            .await
            .into_error(TestError::ApiRequest)?;
        }

        Ok(())
    }
}
