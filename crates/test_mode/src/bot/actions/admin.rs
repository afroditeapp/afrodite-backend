use std::{fmt::Debug, time::Instant};

use api_client::{apis::{media_admin_api, profile_admin_api}, models::ModerationQueueType};
use async_trait::async_trait;
use error_stack::{Result, ResultExt};

use super::{super::super::client::TestError, BotAction, BotState};

#[derive(Debug, Default)]
pub struct AdminBotState {
    profile_content_moderation_started: Option<Instant>,
    profile_text_moderation_started: Option<Instant>,
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
        Self { queue, moderate_all: false }
    }
}

#[async_trait]
impl BotAction for ModerateContentModerationRequest {
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
                    // Test that getting content data works
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

#[derive(Debug)]
pub struct AdminBotContentModerationLogic;

struct EmptyPage;

impl AdminBotContentModerationLogic {
    async fn moderate_one_page(state: &BotState, queue: ModerationQueueType) -> Result<Option<EmptyPage>, TestError> {
        let list = media_admin_api::patch_moderation_request_list(state.api.media(), queue)
            .await
            .change_context(TestError::ApiRequest)?;

        if list.list.is_empty() {
            return Ok(Some(EmptyPage));
        }

        for request in list.list {
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
                // TODO: Check image
                api_client::manual_additions::get_content_fixed(
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
            if start_time.duration_since(previous).as_secs() < config.moderation_session_min_seconds.into() {
                return Ok(());
            }
        }

        state.admin.profile_content_moderation_started = Some(start_time);

        if config.initial_content {
            loop {
                if let Some(EmptyPage) = Self::moderate_one_page(state, ModerationQueueType::InitialMediaModeration).await? {
                    break;
                }

                let current_time = Instant::now();
                if current_time.duration_since(start_time).as_secs() > config.moderation_session_max_seconds.into() {
                    break;
                }
            }
        }

        if config.added_content {
            loop {
                if let Some(EmptyPage) = Self::moderate_one_page(state, ModerationQueueType::MediaModeration).await? {
                    break;
                }

                let current_time = Instant::now();
                if current_time.duration_since(start_time).as_secs() > config.moderation_session_max_seconds.into() {
                    return Ok(());
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct AdminBotProfileTextModerationLogic;

impl AdminBotProfileTextModerationLogic {
    async fn moderate_one_page(state: &BotState) -> Result<Option<EmptyPage>, TestError> {
        let list = profile_admin_api::get_profile_text_pending_moderation_list(state.api.profile(), true)
            .await
            .change_context(TestError::ApiRequest)?;

        if list.values.is_empty() {
            return Ok(Some(EmptyPage));
        }

        for request in list.values {
            profile_admin_api::post_moderate_profile_text(
                state.api.profile(),
                api_client::models::PostModerateProfileText {
                    id: request.id.clone(),
                    text: request.text.clone(),
                    accept: true,
                    rejected_category: None,
                    rejected_details: None,
                },
            )
            .await
            .change_context(TestError::ApiRequest)?;
        }

        Ok(None)
    }
}


#[async_trait]
impl BotAction for AdminBotProfileTextModerationLogic {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let Some(config) = &state.bot_config_file.profile_text_moderation else {
            return Ok(());
        };

        let start_time = Instant::now();

        if let Some(previous) = state.admin.profile_text_moderation_started {
            if start_time.duration_since(previous).as_secs() < config.moderation_session_min_seconds.into() {
                return Ok(());
            }
        }

        state.admin.profile_text_moderation_started = Some(start_time);

        loop {
            if let Some(EmptyPage) = Self::moderate_one_page(state).await? {
                break;
            }

            let current_time = Instant::now();
            if current_time.duration_since(start_time).as_secs() > config.moderation_session_max_seconds.into() {
                return Ok(());
            }
        }

        Ok(())
    }
}
