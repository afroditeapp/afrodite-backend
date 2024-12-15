use std::{fmt::Debug, time::Instant};

use api_client::{apis::media_admin_api, models::{AccountId, ContentId, MediaContentType, ModerationQueueType, ProfileContentModerationRejectedReasonDetails}};
use async_trait::async_trait;
use config::bot_config_file::{NudeDetectionConfig, ProfileContentModerationConfig};
use error_stack::{Result, ResultExt};
use image::DynamicImage;
use super::{BotAction, BotState, EmptyPage};
use crate::client::TestError;

use tracing::error;

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
                        account_id: request.account_id,
                        content_id: request.content_id,
                        accept: true,
                        move_to_human: Some(Some(false)),
                        rejected_category: None,
                        rejected_details: None,
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

struct ContentModerationResult {
    accept: bool,
    move_to_human: bool,
    rejected_details: Option<String>,
}

impl ContentModerationResult {
    fn error() -> Self {
        Self {
            accept: false,
            move_to_human: false,
            rejected_details: Some("Error occurred. Try again and if this continues, please contact customer support.".to_string()),
        }
    }
}

#[derive(Debug)]
pub struct AdminBotContentModerationLogic;

impl AdminBotContentModerationLogic {
    async fn moderate_one_page(
        state: &BotState,
        queue: ModerationQueueType,
        config: &ProfileContentModerationConfig,
    ) -> Result<Option<EmptyPage>, TestError> {
        let list = media_admin_api::get_profile_content_pending_moderation_list(state.api.media(), MediaContentType::JpegImage, queue, true)
            .await
            .change_context(TestError::ApiRequest)?;

        if list.values.is_empty() {
            return Ok(Some(EmptyPage));
        }

        for request in list.values {
            let data = api_client::manual_additions::get_content_fixed(
                state.api.media(),
                &request.account_id.aid,
                &request.content_id.cid,
                false,
            )
            .await
            .change_context(TestError::ApiRequest)?;

            let result = Self::moderate_image(
                data,
                config.nude_detection.clone(),
                &request.account_id,
                &request.content_id,
            ).await;

            media_admin_api::post_moderate_profile_content(
                state.api.media(),
                api_client::models::PostModerateProfileContent {
                    account_id: request.account_id,
                    content_id: request.content_id,
                    accept: result.accept,
                    move_to_human: Some(Some(result.move_to_human)),
                    rejected_category: None,
                    rejected_details: result.rejected_details.map(|v| Some(Box::new(ProfileContentModerationRejectedReasonDetails::new(v)))),
                },
            )
            .await
            .change_context(TestError::ApiRequest)?;
        }

        Ok(None)
    }

    async fn moderate_image(
        data: Vec<u8>,
        nude_config: Option<NudeDetectionConfig>,
        account: &AccountId,
        content: &ContentId,
    ) -> ContentModerationResult {
        let r = tokio::task::spawn_blocking(|| {
            Self::handle_image_sync(data, nude_config)
        })
            .await;

        let log_error = |e: &dyn std::fmt::Debug| error!(
            "Content moderation failed: {e:?}, Account ID: {}, Content ID: {}",
            account.aid,
            content.cid,
        );

        match r {
            Ok(Ok(r)) => r,
            Err(e) => {
                log_error(&e);
                ContentModerationResult::error()
            }
            Ok(Err(e)) => {
                log_error(&e);
                ContentModerationResult::error()
            }
        }
    }

    fn handle_image_sync(
        data: Vec<u8>,
        nude_config: Option<NudeDetectionConfig>,
    ) -> Result<ContentModerationResult, TestError> {
        let img = image::load_from_memory(&data)
            .change_context(TestError::ContentModerationFailed)?;

        if let Some(nude_config) = nude_config {
            if let Some(result) = Self::handle_nude_detection(&img, nude_config)? {
                return Ok(result);
            }
        }

        Ok(ContentModerationResult {
            accept: true,
            move_to_human: false,
            rejected_details: None,
        })
    }

    fn handle_nude_detection(
        img: &DynamicImage,
        nude_config: NudeDetectionConfig,
    ) -> Result<Option<ContentModerationResult>, TestError> {
        let analysis = nude::scan(img).analyse();
        if analysis.nude {
            Ok(Some(ContentModerationResult {
                accept: false,
                move_to_human: nude_config.move_rejected_to_human_moderation,
                rejected_details: Some("Nudity detected".to_string()),
            }))
        } else {
            Ok(None)
        }
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
                    Self::moderate_one_page(state, ModerationQueueType::InitialMediaModeration, config)
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
                    Self::moderate_one_page(state, ModerationQueueType::MediaModeration, config).await?
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
