use std::{fmt::Debug, sync::Arc, time::Instant};

use api_client::{apis::media_admin_api, models::{AccountId, ContentId, MediaContentType, ModerationQueueType, ProfileContentModerationRejectedReasonDetails}};
use async_trait::async_trait;
use config::bot_config_file::{ContentModerationConfig, ModerationAction, NsfwDetectionConfig, NsfwDetectionThresholds, NudeDetectionConfig};
use error_stack::{Result, ResultExt};
use image::DynamicImage;
use nsfw::model::Metric;
use super::{BotAction, BotState, EmptyPage, ModerationResult};
use crate::client::{ApiClient, TestError};

use tracing::error;

#[derive(Debug, Default)]
pub struct ContentModerationState {
    content_moderation_started: Option<Instant>,
    model: Option<Arc<nsfw::Model>>,
}

impl ContentModerationState {
    async fn new(config: &ContentModerationConfig) -> Result<Self, TestError> {
        let config = config.nsfw_detection.clone();
        if let Some(config) = config {
            let model = tokio::task::spawn_blocking(move || {
                let file = std::fs::File::open(config.model_file)
                    .change_context(TestError::ContentModerationFailed)?;
                let model = nsfw::create_model(file)
                    .map_err(|e| TestError::ContentModerationFailed
                        .report()
                        .attach_printable(e.to_string())
                    )?;
                Ok::<_, error_stack::Report<TestError>>(model)
            })
                .await
                .change_context(TestError::ContentModerationFailed)??;
            Ok(Self {
                content_moderation_started: None,
                model: Some(Arc::new(model)),
            })
        } else {
            Ok(Self::default())
        }
    }
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

#[derive(Debug)]
pub struct AdminBotContentModerationLogic;

impl AdminBotContentModerationLogic {
    async fn moderate_one_page(
        api: &ApiClient,
        queue: ModerationQueueType,
        config: &ContentModerationConfig,
        moderation_state: &ContentModerationState,
    ) -> Result<Option<EmptyPage>, TestError> {
        let list = media_admin_api::get_profile_content_pending_moderation_list(api.media(), MediaContentType::JpegImage, queue, true)
            .await
            .change_context(TestError::ApiRequest)?;

        if list.values.is_empty() {
            return Ok(Some(EmptyPage));
        }

        for request in list.values {
            let data = api_client::manual_additions::get_content_fixed(
                api.media(),
                &request.account_id.aid,
                &request.content_id.cid,
                false,
            )
            .await
            .change_context(TestError::ApiRequest)?;

            let result = Self::moderate_image(
                data,
                config.nude_detection.clone(),
                config.nsfw_detection.clone(),
                moderation_state.model.clone(),
                config.default_action,
                &request.account_id,
                &request.content_id,
            ).await;

            media_admin_api::post_moderate_profile_content(
                api.media(),
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
        nsfw_config: Option<NsfwDetectionConfig>,
        nsfw_model: Option<Arc<nsfw::Model>>,
        default_action: ModerationAction,
        account: &AccountId,
        content: &ContentId,
    ) -> ModerationResult {
        let r = tokio::task::spawn_blocking(move || {
            Self::handle_image_sync(data, nude_config, nsfw_config, nsfw_model, default_action)
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
                ModerationResult::error()
            }
            Ok(Err(e)) => {
                log_error(&e);
                ModerationResult::error()
            }
        }
    }

    fn handle_image_sync(
        data: Vec<u8>,
        nude_config: Option<NudeDetectionConfig>,
        nsfw_config: Option<NsfwDetectionConfig>,
        nsfw_model: Option<Arc<nsfw::Model>>,
        default_action: ModerationAction,
    ) -> Result<ModerationResult, TestError> {
        let img = image::load_from_memory(&data)
            .change_context(TestError::ContentModerationFailed)?;

        if let Some(nude_config) = nude_config {
            if let Some(result) = Self::handle_nude_detection(&img, nude_config)? {
                return Ok(result);
            }
        }

        if let (Some(c), Some(m)) = (nsfw_config, nsfw_model) {
            if let Some(result) = Self::handle_nsfw_detection(img, c, &m)? {
                return Ok(result);
            }
        }

        let action = match default_action {
            ModerationAction::Accept => ModerationResult::accept(),
            ModerationAction::Reject => ModerationResult::reject(None),
            ModerationAction::MoveToHuman => ModerationResult::move_to_human(),
        };

        Ok(action)
    }

    fn handle_nude_detection(
        img: &DynamicImage,
        nude_config: NudeDetectionConfig,
    ) -> Result<Option<ModerationResult>, TestError> {
        let analysis = nude::scan(img).analyse();
        if analysis.nude {
            Ok(Some(ModerationResult {
                accept: false,
                move_to_human: nude_config.move_rejected_to_human_moderation,
                rejected_details: Some("Nudity detected. If this is a false positive, please contact customer support.".to_string()),
            }))
        } else {
            Ok(None)
        }
    }

    fn handle_nsfw_detection(
        img: DynamicImage,
        nsfw_config: NsfwDetectionConfig,
        model: &nsfw::Model,
    ) -> Result<Option<ModerationResult>, TestError> {
        let img = img.into_rgba8();
        let results = nsfw::examine(model, &img)
            .map_err(|e| TestError::ContentModerationFailed.report().attach_printable(e.to_string()))?;

        fn threshold(m: &Metric, thresholds: &NsfwDetectionThresholds) -> Option<f32> {
            match m {
                Metric::Drawings => thresholds.drawings,
                Metric::Hentai => thresholds.hentai,
                Metric::Neutral => thresholds.neutral,
                Metric::Porn => thresholds.porn,
                Metric::Sexy => thresholds.sexy,
            }
        }

        if let Some(thresholds) = &nsfw_config.reject {
            for c in &results {
                if let Some(threshold) = threshold(&c.metric, thresholds) {
                    if c.score >= threshold {
                        return Ok(Some(ModerationResult::reject(Some(
                            "NSFW image detected. If this is a false positive, please contact customer support."
                        ))));
                    }
                }
            }
        }

        if let Some(thresholds) = &nsfw_config.move_to_human {
            for c in &results {
                if let Some(threshold) = threshold(&c.metric, thresholds) {
                    if c.score >= threshold {
                        return Ok(Some(ModerationResult::move_to_human()));
                    }
                }
            }
        }

        if let Some(thresholds) = &nsfw_config.accept {
            for c in results {
                if let Some(threshold) = threshold(&c.metric, thresholds) {
                    if c.score >= threshold {
                        return Ok(Some(ModerationResult::accept()));
                    }
                }
            }
        }

        Ok(None)
    }
}

#[async_trait]
impl BotAction for AdminBotContentModerationLogic {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let Some(config) = &state.bot_config_file.content_moderation else {
            return Ok(());
        };

        let moderation_state = if let Some(state) = &mut state.admin.content {
            state
        } else {
            let moderation_state = ContentModerationState::new(config).await?;
            state.admin.content.get_or_insert(moderation_state)
        };

        let start_time = Instant::now();

        if let Some(previous) = moderation_state.content_moderation_started {
            if start_time.duration_since(previous).as_secs()
                < config.moderation_session_min_seconds.into()
            {
                return Ok(());
            }
        }

        moderation_state.content_moderation_started = Some(start_time);

        if config.initial_content {
            loop {
                if let Some(EmptyPage) =
                    Self::moderate_one_page(&state.api, ModerationQueueType::InitialMediaModeration, config, moderation_state)
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
                    Self::moderate_one_page(&state.api, ModerationQueueType::MediaModeration, config, moderation_state).await?
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
