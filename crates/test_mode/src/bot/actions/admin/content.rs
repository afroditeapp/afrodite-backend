use std::{fmt::Debug, sync::Arc, time::Instant};

use api_client::{
    apis::media_admin_api,
    models::{
        MediaContentType, ModerationQueueType, ProfileContentModerationRejectedReasonDetails,
    },
};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
        CreateChatCompletionRequest, ImageUrl,
    },
};
use async_trait::async_trait;
use base64::display::Base64Display;
use config::bot_config_file::{
    ContentModerationConfig, LlmContentModerationConfig, ModerationAction, NsfwDetectionConfig,
    NsfwDetectionThresholds,
};
use error_stack::{Result, ResultExt};
use image::DynamicImage;
use nsfw::model::Metric;
use tracing::{error, info};

use super::{BotAction, BotState, EmptyPage, ModerationResult};
use crate::{
    bot::actions::admin::LlmModerationResult,
    client::{ApiClient, TestError},
};

#[derive(Debug, Default)]
pub struct ContentModerationState {
    content_moderation_started: Option<Instant>,
    model: Option<Arc<nsfw::Model>>,
    client: Option<Client<OpenAIConfig>>,
}

impl ContentModerationState {
    async fn new(config: &ContentModerationConfig) -> Result<Self, TestError> {
        let config = config.nsfw_detection.clone();
        if let Some(config) = config {
            let model = tokio::task::spawn_blocking(move || {
                let file = std::fs::File::open(config.model_file)
                    .change_context(TestError::ContentModerationFailed)?;
                let model = nsfw::create_model(file).map_err(|e| {
                    TestError::ContentModerationFailed
                        .report()
                        .attach_printable(e.to_string())
                })?;
                Ok::<_, error_stack::Report<TestError>>(model)
            })
            .await
            .change_context(TestError::ContentModerationFailed)??;
            Ok(Self {
                content_moderation_started: None,
                model: Some(Arc::new(model)),
                client: None,
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
            let list = media_admin_api::get_profile_content_pending_moderation_list(
                state.api.media(),
                MediaContentType::JpegImage,
                self.queue,
                true,
            )
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
        moderation_state: &mut ContentModerationState,
    ) -> Result<Option<EmptyPage>, TestError> {
        let list = media_admin_api::get_profile_content_pending_moderation_list(
            api.media(),
            MediaContentType::JpegImage,
            queue,
            true,
        )
        .await
        .change_context(TestError::ApiRequest)?;

        if list.values.is_empty() {
            return Ok(Some(EmptyPage));
        }

        for request in list.values {
            let image_data = api_client::manual_additions::get_content_fixed(
                api.media(),
                &request.account_id.aid,
                &request.content_id.cid,
                false,
            )
            .await
            .change_context(TestError::ApiRequest)?;

            let r = Self::handle_image(
                image_data,
                config.nsfw_detection.clone(),
                moderation_state.model.clone(),
                config.llm.as_ref(),
                config.default_action,
                moderation_state,
            )
            .await;

            let result = match r {
                Ok(None) => return Ok(Some(EmptyPage)),
                Ok(Some(r)) => r,
                Err(e) => {
                    error!(
                        "Content moderation failed: {e:?}, Account ID: {}, Content ID: {}",
                        request.account_id, request.content_id,
                    );
                    ModerationResult::error()
                }
            };

            media_admin_api::post_moderate_profile_content(
                api.media(),
                api_client::models::PostModerateProfileContent {
                    account_id: request.account_id,
                    content_id: request.content_id,
                    accept: result.accept,
                    move_to_human: Some(Some(result.move_to_human)),
                    rejected_category: None,
                    rejected_details: result.rejected_details.map(|v| {
                        Some(Box::new(
                            ProfileContentModerationRejectedReasonDetails::new(v),
                        ))
                    }),
                },
            )
            .await
            .change_context(TestError::ApiRequest)?;
        }

        Ok(None)
    }

    async fn handle_image(
        data: Vec<u8>,
        nsfw_config: Option<NsfwDetectionConfig>,
        nsfw_model: Option<Arc<nsfw::Model>>,
        llm_config: Option<&LlmContentModerationConfig>,
        default_action: ModerationAction,
        state: &mut ContentModerationState,
    ) -> Result<Option<ModerationResult>, TestError> {
        let nsfw_result = if let (Some(c), Some(m)) = (nsfw_config, nsfw_model) {
            let img = image::load_from_memory(&data)
                .change_context(TestError::ContentModerationFailed)?;

            tokio::task::spawn_blocking(move || Self::handle_nsfw_detection_sync(img, c, &m))
                .await
                .change_context(TestError::ContentModerationFailed)??
        } else {
            None
        };

        if let Some(nsfw) = &nsfw_result {
            if nsfw.is_rejected() {
                return Ok(nsfw_result);
            }
        }

        let llm_result = if let Some(c) = llm_config {
            match Self::llm_profile_image_moderation(data, c, state).await? {
                LlmModerationResult::StopModerationSesssion => return Ok(None),
                LlmModerationResult::Decision(r) => Some(r),
            }
        } else {
            None
        };

        if let Some(llm) = &llm_result {
            if llm.is_rejected() {
                return Ok(llm_result);
            }
            if llm.is_move_to_human() {
                return Ok(llm_result);
            }
        }

        let action = match default_action {
            ModerationAction::Accept => ModerationResult::accept(),
            ModerationAction::Reject => ModerationResult::reject(None),
            ModerationAction::MoveToHuman => ModerationResult::move_to_human(),
        };

        Ok(nsfw_result.or(llm_result).or(Some(action)))
    }

    fn handle_nsfw_detection_sync(
        img: DynamicImage,
        nsfw_config: NsfwDetectionConfig,
        model: &nsfw::Model,
    ) -> Result<Option<ModerationResult>, TestError> {
        let img = img.into_rgba8();
        let results = nsfw::examine(model, &img).map_err(|e| {
            TestError::ContentModerationFailed
                .report()
                .attach_printable(e.to_string())
        })?;

        if nsfw_config.debug_log_results {
            info!("NSFW detection results: {:?}", &results);
        }

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
                            "NSFW image detected. If this is a false positive, please contact customer support.",
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

    async fn llm_profile_image_moderation(
        image_data: Vec<u8>,
        config: &LlmContentModerationConfig,
        state: &mut ContentModerationState,
    ) -> Result<LlmModerationResult, TestError> {
        let client = state.client.get_or_insert_with(|| {
            Client::with_config(
                OpenAIConfig::new()
                    .with_api_base(config.openai_api_url.to_string())
                    .with_api_key(""),
            )
        });

        let expected_response_lowercase = config.expected_response.to_lowercase();

        let image = ChatCompletionRequestMessageContentPartImage {
            image_url: ImageUrl {
                url: format!(
                    "data:image/jpeg;base64,{}",
                    Base64Display::new(&image_data, &base64::engine::general_purpose::STANDARD),
                ),
                detail: None,
            },
        };

        let message = ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Array(vec![image.into()]),
            name: None,
        };

        // Hide warning about max_tokens as Ollama does not yet
        // support max_completion_tokens.
        #[allow(deprecated)]
        let r = client
            .chat()
            .create(CreateChatCompletionRequest {
                messages: vec![
                    ChatCompletionRequestMessage::System(config.system_text.clone().into()),
                    ChatCompletionRequestMessage::User(message),
                ],
                model: config.model.clone(),
                temperature: Some(0.0),
                seed: Some(0),
                max_completion_tokens: Some(config.max_tokens),
                max_tokens: Some(config.max_tokens),
                ..Default::default()
            })
            .await;
        let response = match r.map(|r| r.choices.into_iter().next()) {
            Ok(Some(r)) => match r.message.content {
                Some(response) => response,
                None => {
                    error!("Content moderation error: no response content from LLM");
                    return Ok(LlmModerationResult::StopModerationSesssion);
                }
            },
            Ok(None) => {
                error!("Content moderation error: no response from LLM");
                return Ok(LlmModerationResult::StopModerationSesssion);
            }
            Err(e) => {
                error!("Content moderation error: {}", e);
                return Ok(LlmModerationResult::StopModerationSesssion);
            }
        };

        let response_lowercase = response.trim().to_lowercase();
        let response_first_line = response_lowercase.lines().next().unwrap_or_default();
        let accepted = response_lowercase.starts_with(&expected_response_lowercase)
            || response_first_line.contains(&expected_response_lowercase);
        if config.debug_log_results {
            info!("LLM content moderation result: '{}'", response);
        }
        let rejected_details = if !accepted && config.debug_show_llm_output_when_rejected {
            Some(response)
        } else {
            None
        };

        let move_to_human = (accepted && config.move_accepted_to_human_moderation)
            || (!accepted && config.move_rejected_to_human_moderation);

        Ok(LlmModerationResult::Decision(ModerationResult {
            accept: accepted,
            rejected_details,
            move_to_human,
        }))
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
                if let Some(EmptyPage) = Self::moderate_one_page(
                    &state.api,
                    ModerationQueueType::InitialMediaModeration,
                    config,
                    moderation_state,
                )
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
                if let Some(EmptyPage) = Self::moderate_one_page(
                    &state.api,
                    ModerationQueueType::MediaModeration,
                    config,
                    moderation_state,
                )
                .await?
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
