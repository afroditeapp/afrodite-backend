use std::fmt::Debug;

use api_client::apis::media_api;
use async_trait::async_trait;

use error_stack::Result;

use super::{super::super::client::TestError, BotAction};

use crate::utils::IntoReportExt;

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
            let images = [
                Some(request.content.image1),
                request.content.image2.flatten(),
                request.content.image3.flatten(),
            ];
            for content_id in images.iter().flatten() {
                api_client::manual_additions::get_image_fixed(
                    state.api.media(),
                    &request.request_creator_id.to_string(),
                    &content_id.to_string(),
                )
                .await
                .into_error(TestError::ApiRequest)?;
            }
            media_api::post_handle_moderation_request(
                state.api.media(),
                &request.request_creator_id.to_string(),
                api_client::models::HandleModerationRequest { accept: true },
            )
            .await
            .into_error(TestError::ApiRequest)?;
        }

        Ok(())
    }
}
