use std::fmt::Debug;

use api_client::{apis::media_admin_api, models::ModerationQueueType};
use async_trait::async_trait;
use error_stack::{Result, ResultExt};

use super::{super::super::client::TestError, BotAction, BotState};

#[derive(Debug)]
pub struct ModerateMediaModerationRequest {
    queue: ModerationQueueType,
    moderate_all: bool,
}

impl ModerateMediaModerationRequest {
    pub const fn moderate_initial_content() -> Self {
        Self {
            queue: ModerationQueueType::InitialMediaModeration,
            moderate_all: false,
        }
    }

    pub const fn moderate_all_initial_content() -> Self {
        Self {
            queue: ModerationQueueType::InitialMediaModeration,
            moderate_all: true,
        }
    }

    pub const fn moderate_additional_content() -> Self {
        Self {
            queue: ModerationQueueType::MediaModeration,
            moderate_all: false,
        }
    }

    pub const fn from_queue(queue: ModerationQueueType) -> Self {
        Self { queue, moderate_all: false }
    }
}

#[async_trait]
impl BotAction for ModerateMediaModerationRequest {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        loop {
            let list = media_admin_api::patch_moderation_request_list(state.api.media(), self.queue)
                .await
                .change_context(TestError::ApiRequest)?;

            for request in list.list.clone() {
                let images = [
                    Some(request.content.c0),
                    request.content.c1.flatten(),
                    request.content.c2.flatten(),
                    request.content.c3.flatten(),
                    request.content.c4.flatten(),
                    request.content.c5.flatten(),
                    request.content.c6.flatten(),
                ];
                for content_id in images.iter().flatten() {
                    api_client::manual_additions::get_content_fixed(
                        state.api.media(),
                        &request.request_creator_id.to_string(),
                        &content_id.to_string(),
                        false,
                    )
                    .await
                    .change_context(TestError::ApiRequest)
                    // This logging exists because this request failed
                    // when GetProfileList benchmark was running.
                    // When the error was noticed there was multiple
                    // admin bots moderating.
                    .attach_printable_lazy(|| format!(
                        "Request creator: {}, Content ID: {}",
                        request.request_creator_id,
                        content_id,
                    ))?;
                }
                media_admin_api::post_handle_moderation_request(
                    state.api.media(),
                    &request.request_creator_id.to_string(),
                    api_client::models::HandleModerationRequest { accept: true },
                )
                .await
                .change_context(TestError::ApiRequest)?;
            }

            if !self.moderate_all || list.list.is_empty() {
                break
            }
        }
        Ok(())
    }
}
