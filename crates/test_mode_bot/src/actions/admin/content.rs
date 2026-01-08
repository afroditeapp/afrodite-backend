use std::{fmt::Debug, sync::Arc};

use api_client::{
    apis::{media_admin_api, media_api},
    models::{
        MediaContentModerationRejectedReasonDetails, MediaContentPendingModeration,
        MediaContentType, ModerationQueueType,
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
use futures::stream::{self, StreamExt};
use image::DynamicImage;
use nsfw::model::Metric;
use test_mode_utils::client::{ApiClient, TestError};
use tracing::info;

use super::{BotAction, BotState, EmptyPage, ModerationResult};

#[derive(Debug, Clone)]
struct NsfwConfigAndModel {
    config: Arc<NsfwDetectionConfig>,
    model: Arc<nsfw::Model>,
}

#[derive(Debug, Clone)]
struct LlmConfigAndClient {
    config: Arc<LlmContentModerationConfig>,
    client: Client<OpenAIConfig>,
}

#[derive(Debug)]
pub struct ContentModerationState {
    nsfw: Option<NsfwConfigAndModel>,
    llm_primary: Option<LlmConfigAndClient>,
    llm_secondary: Option<LlmConfigAndClient>,
}

impl ContentModerationState {
    pub async fn new(
        config: &ContentModerationConfig,
        reqwest_client: reqwest::Client,
    ) -> Result<Self, TestError> {
        let model = if let Some(config) = config.nsfw_detection.clone() {
            let model_file = config.model_file.clone();
            let model = tokio::task::spawn_blocking(move || {
                let file = std::fs::File::open(model_file)
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
            Some(NsfwConfigAndModel {
                config: config.into(),
                model: model.into(),
            })
        } else {
            None
        };

        let llm_primary = config
            .llm_primary
            .as_ref()
            .map(|config| LlmConfigAndClient {
                client: Client::with_config(
                    OpenAIConfig::new()
                        .with_api_base(config.openai_api_url.to_string())
                        .with_api_key(""),
                )
                .with_http_client(reqwest_client.clone()),
                config: config.clone().into(),
            });

        let llm_secondary = config
            .llm_secondary
            .as_ref()
            .map(|config| LlmConfigAndClient {
                client: Client::with_config(
                    OpenAIConfig::new()
                        .with_api_base(config.openai_api_url.to_string())
                        .with_api_key(""),
                )
                .with_http_client(reqwest_client.clone()),
                config: config.clone().into(),
            });

        Ok(Self {
            nsfw: model,
            llm_primary,
            llm_secondary,
        })
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

    pub const fn from_queue(queue: ModerationQueueType, moderate_all: bool) -> Self {
        Self {
            queue,
            moderate_all,
        }
    }
}

#[async_trait]
impl BotAction for ModerateContentModerationRequest {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        loop {
            let list = media_admin_api::get_media_content_pending_moderation_list(
                state.api(),
                MediaContentType::JpegImage,
                self.queue,
                true,
            )
            .await
            .change_context(TestError::ApiRequest)?;

            for request in list.values.clone() {
                // Test that getting content data works
                media_api::get_content(
                    state.api(),
                    &request.account_id.to_string(),
                    &request.content_id.to_string(),
                    Some(false),
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
                })?
                .bytes()
                .await
                .change_context(TestError::ApiRequest)?
                .to_vec();

                media_admin_api::post_moderate_media_content(
                    state.api(),
                    api_client::models::PostModerateMediaContent {
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
        state: &mut ContentModerationState,
    ) -> Result<Option<EmptyPage>, TestError> {
        let list = media_admin_api::get_media_content_pending_moderation_list(
            api.api(),
            MediaContentType::JpegImage,
            queue,
            true,
        )
        .await
        .change_context(TestError::ApiRequest)?;

        if list.values.is_empty() {
            return Ok(Some(EmptyPage));
        }

        let mut stream = stream::iter(list.values)
            .map(|moderation| {
                let api = api.clone();
                let nsfw = state.nsfw.clone();
                let llm_primary = state.llm_primary.clone();
                let llm_secondary = state.llm_secondary.clone();
                async move {
                    Self::handle_pending_moderation(
                        &api,
                        config,
                        nsfw,
                        llm_primary,
                        llm_secondary,
                        moderation,
                    )
                    .await
                }
            })
            .buffer_unordered(config.concurrency().into());

        loop {
            match stream.next().await {
                Some(Ok(())) => (),
                Some(Err(e)) => return Err(e),
                None => return Ok(None),
            }
        }
    }

    async fn handle_pending_moderation(
        api: &ApiClient,
        config: &ContentModerationConfig,
        nsfw: Option<NsfwConfigAndModel>,
        llm_primary: Option<LlmConfigAndClient>,
        llm_secondary: Option<LlmConfigAndClient>,
        moderation: MediaContentPendingModeration,
    ) -> Result<(), TestError> {
        let image_data = media_api::get_content(
            api.api(),
            &moderation.account_id.aid,
            &moderation.content_id.cid,
            Some(false),
        )
        .await
        .change_context(TestError::ApiRequest)?
        .bytes()
        .await
        .change_context(TestError::ApiRequest)?
        .to_vec();

        let result = Self::handle_image(
            image_data,
            nsfw,
            llm_primary,
            llm_secondary,
            config.default_action,
        )
        .await?;

        if result.delete {
            media_api::delete_content(
                api.api(),
                &moderation.account_id.aid,
                &moderation.content_id.cid,
            )
            .await
            .change_context(TestError::ApiRequest)?;

            if config.debug_log_delete {
                info!("Image deleted");
            }
        } else {
            media_admin_api::post_moderate_media_content(
                api.api(),
                api_client::models::PostModerateMediaContent {
                    account_id: moderation.account_id,
                    content_id: moderation.content_id,
                    accept: result.accept,
                    move_to_human: Some(Some(result.move_to_human)),
                    rejected_category: None,
                    rejected_details: Some(result.rejected_details.and_then(|v| {
                        if v.is_empty() {
                            None
                        } else {
                            Some(Box::new(MediaContentModerationRejectedReasonDetails::new(
                                v,
                            )))
                        }
                    })),
                },
            )
            .await
            .change_context(TestError::ApiRequest)?;
        }

        Ok(())
    }

    async fn handle_image(
        data: Vec<u8>,
        nsfw: Option<NsfwConfigAndModel>,
        llm_primary: Option<LlmConfigAndClient>,
        llm_secondary: Option<LlmConfigAndClient>,
        default_action: ModerationAction,
    ) -> Result<ModerationResult, TestError> {
        let nsfw_result = if let Some(nsfw) = nsfw {
            let img = image::load_from_memory(&data)
                .change_context(TestError::ContentModerationFailed)?;

            tokio::task::spawn_blocking(move || Self::handle_nsfw_detection_sync(img, nsfw))
                .await
                .change_context(TestError::ContentModerationFailed)??
        } else {
            None
        };

        if let Some(nsfw) = &nsfw_result
            && nsfw.is_deleted_or_rejected()
        {
            return Ok(nsfw.clone());
        }

        let llm_result = if let Some(primary) = llm_primary {
            match Self::llm_profile_image_moderation_and_retry(&data, primary).await? {
                None => {
                    if let Some(secondary) = llm_secondary {
                        Self::llm_profile_image_moderation_and_retry(&data, secondary).await?
                    } else {
                        None
                    }
                }
                Some(r) => Some(r),
            }
        } else {
            None
        };

        if let Some(llm) = &llm_result {
            if llm.is_deleted_or_rejected() {
                return Ok(llm.clone());
            }
            if llm.is_move_to_human() {
                return Ok(llm.clone());
            }
        }

        let r = nsfw_result
            .or(llm_result)
            .unwrap_or_else(|| match default_action {
                ModerationAction::Accept => ModerationResult::accept(),
                ModerationAction::Reject => ModerationResult::reject(None),
                ModerationAction::MoveToHuman => ModerationResult::move_to_human(),
            });

        Ok(r)
    }

    fn handle_nsfw_detection_sync(
        img: DynamicImage,
        nsfw: NsfwConfigAndModel,
    ) -> Result<Option<ModerationResult>, TestError> {
        let img = img.into_rgba8();
        let results = nsfw::examine(&nsfw.model, &img).map_err(|e| {
            TestError::ContentModerationFailed
                .report()
                .attach_printable(e.to_string())
        })?;

        if nsfw.config.debug_log_results {
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

        if let Some(thresholds) = &nsfw.config.delete {
            for c in &results {
                if let Some(threshold) = threshold(&c.metric, thresholds)
                    && c.score >= threshold
                {
                    return Ok(Some(ModerationResult::delete()));
                }
            }
        }

        if let Some(thresholds) = &nsfw.config.reject {
            for c in &results {
                if let Some(threshold) = threshold(&c.metric, thresholds)
                    && c.score >= threshold
                {
                    return Ok(Some(ModerationResult::reject(Some(
                        "NSFW image detected. If this is a false positive, please contact customer support.",
                    ))));
                }
            }
        }

        if let Some(thresholds) = &nsfw.config.move_to_human {
            for c in &results {
                if let Some(threshold) = threshold(&c.metric, thresholds)
                    && c.score >= threshold
                {
                    return Ok(Some(ModerationResult::move_to_human()));
                }
            }
        }

        if let Some(thresholds) = &nsfw.config.accept {
            for c in results {
                if let Some(threshold) = threshold(&c.metric, thresholds)
                    && c.score >= threshold
                {
                    return Ok(Some(ModerationResult::accept()));
                }
            }
        }

        Ok(None)
    }

    async fn llm_profile_image_moderation(
        image_data: &[u8],
        llm: &LlmConfigAndClient,
    ) -> Result<Option<ModerationResult>, TestError> {
        let config = &llm.config;
        let expected_response_lowercase = llm.config.expected_response.to_lowercase();

        let image = ChatCompletionRequestMessageContentPartImage {
            image_url: ImageUrl {
                url: format!(
                    "data:image/jpeg;base64,{}",
                    Base64Display::new(image_data, &base64::engine::general_purpose::STANDARD),
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
        let r = llm
            .client
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
                    return Err(TestError::LlmError).attach_printable(
                        "LLM image moderation error: no response content".to_string(),
                    );
                }
            },
            Ok(None) => {
                return Err(TestError::LlmError)
                    .attach_printable("LLM image moderation error: no response".to_string());
            }
            Err(e) => {
                return Err(TestError::LlmError)
                    .attach_printable(format!("LLM image moderation failed: {e}"));
            }
        };

        let response_lowercase = response.trim().to_lowercase();
        let response_first_line = response_lowercase.lines().next().unwrap_or_default();
        let accepted = response_lowercase.starts_with(&expected_response_lowercase)
            || response_first_line.contains(&expected_response_lowercase);
        if config.debug_log_results {
            info!("LLM image moderation result: '{}'", response);
        }
        let rejected_details = if !accepted && config.add_llm_output_to_rejection_details {
            Some(response)
        } else {
            None
        };

        if config.delete_accepted && accepted {
            return Ok(Some(ModerationResult::delete()));
        }

        if config.ignore_rejected && !accepted {
            return Ok(None);
        }

        let move_to_human = (accepted && config.move_accepted_to_human_moderation)
            || (!accepted && config.move_rejected_to_human_moderation);

        Ok(Some(ModerationResult {
            accept: accepted,
            rejected_details,
            move_to_human,
            delete: false,
        }))
    }

    async fn llm_profile_image_moderation_and_retry(
        image_data: &[u8],
        llm: LlmConfigAndClient,
    ) -> Result<Option<ModerationResult>, TestError> {
        let retry_wait_times = &llm.config.retry_wait_times_in_seconds;
        let mut attempt = 0;

        loop {
            match Self::llm_profile_image_moderation(image_data, &llm).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt < retry_wait_times.len() {
                        let wait_time = retry_wait_times[attempt];
                        info!(
                            "LLM image moderation attempt {} failed, retrying in {} seconds...",
                            attempt + 1,
                            wait_time
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(wait_time as u64))
                            .await;
                        attempt += 1;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }
}

impl AdminBotContentModerationLogic {
    pub async fn run_content_moderation(
        api: &ApiClient,
        config: &ContentModerationConfig,
        moderation_state: &mut ContentModerationState,
    ) -> Result<(), TestError> {
        if config.initial_content {
            loop {
                if let Some(EmptyPage) = Self::moderate_one_page(
                    api,
                    ModerationQueueType::InitialMediaModeration,
                    config,
                    moderation_state,
                )
                .await?
                {
                    break;
                }
            }
        }

        if config.added_content {
            loop {
                if let Some(EmptyPage) = Self::moderate_one_page(
                    api,
                    ModerationQueueType::MediaModeration,
                    config,
                    moderation_state,
                )
                .await?
                {
                    break;
                }
            }
        }

        Ok(())
    }
}
