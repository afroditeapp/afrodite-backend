use std::{fmt::Debug, time::Instant};

use api_client::{apis::media_admin_api, models::{MediaContentType, ModerationQueueType}};
use async_trait::async_trait;
use error_stack::{Result, ResultExt};
use profile_text::ProfileTextModerationState;

use super::{super::super::client::TestError, BotAction, BotState};

pub mod profile_text;

#[derive(Debug, Default)]
pub struct AdminBotState {
    profile_content_moderation_started: Option<Instant>,
    profile_text: Option<ProfileTextModerationState>,
}

#[derive(Debug)]
pub struct ModerateContentModerationRequest {
    queue: ModerationQueueType,
    moderate_all: bool,
}

impl ModerateContentModerationRequest {
    pub const fn moderate_all_initial_content() -> Self {
        Self {
            queue: ModerationQueueType::InitialMediaModeration,
            moderate_all: true,
        }
    }

    pub const fn from_queue(queue: ModerationQueueType) -> Self {
        Self {
            queue,
            moderate_all: false,
        }
    }
}

#[async_trait]
impl BotAction for ModerateContentModerationRequest {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        loop {
            let list =
                media_admin_api::get_profile_content_pending_moderation_list(state.api.media(), MediaContentType::JpegImage, self.queue, true)
                    .await
                    .change_context(TestError::ApiRequest)?;

            for request in list.values.clone() {
                // Test that getting content data works
                api_client::manual_additions::get_content_fixed(
                    state.api.media(),
                    &request.account_id.to_string(),
                    &request.content_id.to_string(),
                    false,
                )
                .await
                .change_context(TestError::ApiRequest)
                // This logging exists because this request failed
                // when GetProfileList benchmark was running.
                // When the error was noticed there was multiple
                // admin bots moderating.
                .attach_printable_lazy(|| {
                    format!(
                        "Request creator: {}, Content ID: {}",
                        request.account_id, request.content_id,
                    )
                })?;

                media_admin_api::post_moderate_profile_content(
                    state.api.media(),
                    api_client::models::PostModerateProfileContent {
                        content_id: request.content_id,
                        accept: true,
                        move_to_human: Some(Some(false)),
                        rejected_category: None,
                        rejected_details: None,
                        text: "".to_string(),
                    },
                )
                .await
                .change_context(TestError::ApiRequest)?;
            }

            if !self.moderate_all || list.values.is_empty() {
                break;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct AdminBotContentModerationLogic;

struct EmptyPage;

impl AdminBotContentModerationLogic {
    async fn moderate_one_page(
        state: &BotState,
        queue: ModerationQueueType,
    ) -> Result<Option<EmptyPage>, TestError> {
        let list = media_admin_api::get_profile_content_pending_moderation_list(state.api.media(), MediaContentType::JpegImage, queue, true)
            .await
            .change_context(TestError::ApiRequest)?;

        if list.values.is_empty() {
            return Ok(Some(EmptyPage));
        }

        for request in list.values {
            // TODO: Check image
            api_client::manual_additions::get_content_fixed(
                state.api.media(),
                &request.account_id.to_string(),
                &request.content_id.to_string(),
                false,
            )
            .await
            .change_context(TestError::ApiRequest)?;

            media_admin_api::post_moderate_profile_content(
                state.api.media(),
                api_client::models::PostModerateProfileContent {
                    content_id: request.content_id,
                    accept: true,
                    move_to_human: Some(Some(false)),
                    rejected_category: None,
                    rejected_details: None,
                    text: "".to_string(),
                },
            )
            .await
            .change_context(TestError::ApiRequest)?;
        }

        Ok(None)
    }
}

#[async_trait]
impl BotAction for AdminBotContentModerationLogic {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let Some(config) = &state.bot_config_file.profile_content_moderation else {
            return Ok(());
        };

        let start_time = Instant::now();

        if let Some(previous) = state.admin.profile_content_moderation_started {
            if start_time.duration_since(previous).as_secs()
                < config.moderation_session_min_seconds.into()
            {
                return Ok(());
            }
        }

        state.admin.profile_content_moderation_started = Some(start_time);

        if config.initial_content {
            loop {
                if let Some(EmptyPage) =
                    Self::moderate_one_page(state, ModerationQueueType::InitialMediaModeration)
                        .await?
                {
                    break;
                }

                let current_time = Instant::now();
                if current_time.duration_since(start_time).as_secs()
                    > config.moderation_session_max_seconds.into()
                {
                    break;
                }
            }
        }

        if config.added_content {
            loop {
                if let Some(EmptyPage) =
                    Self::moderate_one_page(state, ModerationQueueType::MediaModeration).await?
                {
                    break;
                }

                let current_time = Instant::now();
                if current_time.duration_since(start_time).as_secs()
                    > config.moderation_session_max_seconds.into()
                {
                    return Ok(());
                }
            }
        }

        Ok(())
    }
}
