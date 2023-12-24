use std::fmt::Debug;

use api_client::apis::media_admin_api;
use async_trait::async_trait;
use error_stack::{Result, ResultExt};

use super::{super::super::client::TestError, BotAction, BotState};

#[derive(Debug)]
pub struct ModerateMediaModerationRequest;

#[async_trait]
impl BotAction for ModerateMediaModerationRequest {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let list = media_admin_api::patch_moderation_request_list(state.api.media())
            .await
            .change_context(TestError::ApiRequest)?;

        for request in list.list {
            let images = [
                request.content.initial_moderation_security_image.flatten(),
                Some(request.content.content1),
                request.content.content2.flatten(),
                request.content.content3.flatten(),
                request.content.content4.flatten(),
                request.content.content5.flatten(),
                request.content.content6.flatten(),
            ];
            for content_id in images.iter().flatten() {
                api_client::manual_additions::get_image_fixed(
                    state.api.media(),
                    &request.request_creator_id.to_string(),
                    &content_id.to_string(),
                    false,
                )
                .await
                .change_context(TestError::ApiRequest)?;
            }
            media_admin_api::post_handle_moderation_request(
                state.api.media(),
                &request.request_creator_id.to_string(),
                api_client::models::HandleModerationRequest { accept: true },
            )
            .await
            .change_context(TestError::ApiRequest)?;
        }

        Ok(())
    }
}
