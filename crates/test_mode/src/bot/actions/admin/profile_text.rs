use std::{fmt::Debug, time::Instant};

use api_client::{
    apis::profile_admin_api,
    models::{ProfileModerationRejectedReasonDetails, ProfileStringModerationContentType},
};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequest},
};
use async_trait::async_trait;
use config::bot_config_file::{
    LlmStringModerationConfig, ModerationAction, ProfileStringModerationConfig,
};
use error_stack::{Result, ResultExt};
use tracing::{error, info};
use unicode_segmentation::UnicodeSegmentation;

use super::{BotAction, BotState, EmptyPage, ModerationResult};
use crate::{
    bot::actions::admin::LlmModerationResult,
    client::{ApiClient, TestError},
};

#[derive(Debug)]
pub struct ProfileTextModerationState {
    moderation_started: Option<Instant>,
    client: Option<Client<OpenAIConfig>>,
    reqwest_client: reqwest::Client,
}

#[derive(Debug)]
pub struct AdminBotProfileStringModerationLogic {
    content_type: ProfileStringModerationContentType,
}

impl AdminBotProfileStringModerationLogic {
    pub const fn profile_name() -> Self {
        Self {
            content_type: ProfileStringModerationContentType::ProfileName,
        }
    }

    pub const fn profile_text() -> Self {
        Self {
            content_type: ProfileStringModerationContentType::ProfileText,
        }
    }

    async fn moderate_one_page(
        &self,
        api: &ApiClient,
        config: &ProfileStringModerationConfig,
        state: &mut ProfileTextModerationState,
    ) -> Result<Option<EmptyPage>, TestError> {
        let list = profile_admin_api::get_profile_string_pending_moderation_list(
            api.profile(),
            self.content_type,
            true,
        )
        .await
        .change_context(TestError::ApiRequest)?;

        if list.values.is_empty() {
            return Ok(Some(EmptyPage));
        }

        for request in list.values {
            // Allow texts with only single visible character
            if config.accept_single_visible_character && request.value.graphemes(true).count() == 1
            {
                // Ignore errors as the user might have changed the text to
                // another one or it is already moderated.
                let _ = profile_admin_api::post_moderate_profile_string(
                    api.profile(),
                    api_client::models::PostModerateProfileString {
                        content_type: self.content_type,
                        id: request.id.clone(),
                        value: request.value.clone(),
                        accept: true,
                        rejected_category: None,
                        rejected_details: Box::default(),
                        move_to_human: None,
                    },
                )
                .await;

                continue;
            }

            let r = if let Some(llm_config) = &config.llm {
                let r =
                    Self::llm_profile_text_moderation(&request.value, llm_config, state).await?;

                match r {
                    LlmModerationResult::StopModerationSesssion => return Ok(Some(EmptyPage)),
                    LlmModerationResult::Decision(r) => r,
                }
            } else {
                None
            };

            let r = r.unwrap_or_else(|| match config.default_action {
                ModerationAction::Accept => ModerationResult::accept(),
                ModerationAction::Reject => ModerationResult::reject(None),
                ModerationAction::MoveToHuman => ModerationResult::move_to_human(),
            });

            // Ignore errors as the user might have changed the text to
            // another one or it is already moderated.
            let _ = profile_admin_api::post_moderate_profile_string(
                api.profile(),
                api_client::models::PostModerateProfileString {
                    content_type: self.content_type,
                    id: request.id.clone(),
                    value: request.value.clone(),
                    accept: r.accept,
                    rejected_category: None,
                    rejected_details: Box::new(ProfileModerationRejectedReasonDetails::new(
                        r.rejected_details.unwrap_or_default(),
                    )),
                    move_to_human: if r.move_to_human {
                        Some(Some(true))
                    } else {
                        None
                    },
                },
            )
            .await;
        }

        Ok(None)
    }

    async fn llm_profile_text_moderation(
        profile_text: &str,
        config: &LlmStringModerationConfig,
        state: &mut ProfileTextModerationState,
    ) -> Result<LlmModerationResult, TestError> {
        let client = state.client.get_or_insert_with(|| {
            Client::with_config(
                OpenAIConfig::new()
                    .with_api_base(config.openai_api_url.to_string())
                    .with_api_key(""),
            )
            .with_http_client(state.reqwest_client.clone())
        });

        let expected_response_lowercase = config.expected_response.to_lowercase();
        let profile_text_paragraph = profile_text.lines().collect::<Vec<&str>>().join(" ");

        let user_text = config.user_text_template.replace(
            ProfileStringModerationConfig::TEMPLATE_PLACEHOLDER_TEXT,
            &profile_text_paragraph,
        );

        // Hide warning about max_tokens as Ollama does not yet
        // support max_completion_tokens.
        #[allow(deprecated)]
        let r = client
            .chat()
            .create(CreateChatCompletionRequest {
                messages: vec![
                    ChatCompletionRequestMessage::System(config.system_text.clone().into()),
                    ChatCompletionRequestMessage::User(user_text.into()),
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
                    error!("Profile text moderation error: no response content from LLM");
                    return Ok(LlmModerationResult::StopModerationSesssion);
                }
            },
            Ok(None) => {
                error!("Profile text moderation error: no response from LLM");
                return Ok(LlmModerationResult::StopModerationSesssion);
            }
            Err(e) => {
                error!("Profile text moderation error: {}", e);
                return Ok(LlmModerationResult::StopModerationSesssion);
            }
        };

        let response_lowercase = response.trim().to_lowercase();
        let response_first_line = response_lowercase.lines().next().unwrap_or_default();
        let accepted = response_lowercase.starts_with(&expected_response_lowercase)
            || response_first_line.contains(&expected_response_lowercase);
        if config.debug_log_results {
            info!("LLM text moderation result: '{}'", response);
        }
        let rejected_details = if !accepted && config.debug_show_llm_output_when_rejected {
            Some(response)
        } else {
            None
        };

        let move_to_human = !accepted && config.move_rejected_to_human_moderation;

        Ok(LlmModerationResult::Decision(Some(ModerationResult {
            accept: accepted,
            rejected_details,
            move_to_human,
            delete: false,
        })))
    }
}

#[async_trait]
impl BotAction for AdminBotProfileStringModerationLogic {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let config = match self.content_type {
            ProfileStringModerationContentType::ProfileName => {
                &state.bot_config_file.profile_name_moderation
            }
            ProfileStringModerationContentType::ProfileText => {
                &state.bot_config_file.profile_text_moderation
            }
        };

        let Some(config) = config else {
            return Ok(());
        };

        let moderation_state =
            state
                .admin
                .profile_text
                .get_or_insert_with(|| ProfileTextModerationState {
                    moderation_started: None,
                    client: None,
                    reqwest_client: state.reqwest_client.clone(),
                });

        let start_time = Instant::now();

        if let Some(previous) = moderation_state.moderation_started {
            if start_time.duration_since(previous).as_secs()
                < config.moderation_session_min_seconds.into()
            {
                return Ok(());
            }
        }

        moderation_state.moderation_started = Some(start_time);

        loop {
            if let Some(EmptyPage) = self
                .moderate_one_page(&state.api, config, moderation_state)
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

        Ok(())
    }
}
