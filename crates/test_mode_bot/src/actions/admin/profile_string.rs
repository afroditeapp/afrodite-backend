use std::{fmt::Debug, sync::Arc};

use api_client::{
    apis::profile_admin_api,
    models::{
        ProfileStringModerationContentType, ProfileStringModerationRejectedReasonDetails,
        ProfileStringPendingModeration,
    },
};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequest},
};
use config::bot_config_file::{
    LlmStringModerationConfig, ModerationAction, ProfileStringModerationConfig,
};
use error_stack::{Result, ResultExt};
use futures::{StreamExt, stream};
use test_mode_utils::client::{ApiClient, TestError};
use tracing::info;
use unicode_segmentation::UnicodeSegmentation;

use super::{EmptyPage, ModerationResult};

#[derive(Debug, Clone)]
struct LlmConfigAndClient {
    config: Arc<LlmStringModerationConfig>,
    client: Client<OpenAIConfig>,
}

#[derive(Debug)]
pub struct ProfileStringModerationState {
    llm: Option<LlmConfigAndClient>,
}

impl ProfileStringModerationState {
    pub fn new(config: &ProfileStringModerationConfig, reqwest_client: reqwest::Client) -> Self {
        let llm = config.llm.as_ref().map(|config| LlmConfigAndClient {
            client: Client::with_config(
                OpenAIConfig::new()
                    .with_api_base(config.openai_api_url.to_string())
                    .with_api_key(""),
            )
            .with_http_client(reqwest_client.clone()),
            config: config.clone().into(),
        });

        Self { llm }
    }
}

#[derive(Debug)]
pub struct AdminBotProfileStringModerationLogic {
    content_type: ProfileStringModerationContentType,
}

impl AdminBotProfileStringModerationLogic {
    async fn moderate_one_page(
        &self,
        api: &ApiClient,
        config: &ProfileStringModerationConfig,
        state: &mut ProfileStringModerationState,
    ) -> Result<Option<EmptyPage>, TestError> {
        let list = profile_admin_api::get_profile_string_pending_moderation_list(
            api.api(),
            self.content_type,
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
                let llm = state.llm.clone();
                async move {
                    Self::handle_pending_moderation(
                        &api,
                        config,
                        llm,
                        self.content_type,
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
        config: &ProfileStringModerationConfig,
        llm: Option<LlmConfigAndClient>,
        content_type: ProfileStringModerationContentType,
        moderation: ProfileStringPendingModeration,
    ) -> Result<(), TestError> {
        // Allow texts with only single visible character
        if config.accept_single_visible_character && moderation.value.graphemes(true).count() == 1 {
            // Ignore errors as the user might have changed the text to
            // another one or it is already moderated.
            let _ = profile_admin_api::post_moderate_profile_string(
                api.api(),
                api_client::models::PostModerateProfileString {
                    content_type,
                    id: moderation.id.clone(),
                    value: moderation.value.clone(),
                    accept: true,
                    rejected_category: None,
                    rejected_details: None,
                    move_to_human: None,
                },
            )
            .await;

            return Ok(());
        }

        let r = if let Some(llm) = llm {
            Self::llm_profile_string_moderation_and_retry(&moderation.value, llm, content_type)
                .await?
        } else {
            None
        };

        let r = r.unwrap_or_else(|| match config.default_action {
            ModerationAction::Accept => ModerationResult::accept(),
            ModerationAction::Reject => ModerationResult::reject(None),
            ModerationAction::MoveToHuman => ModerationResult::move_to_human(None),
        });

        // Ignore errors as the user might have changed the text to
        // another one or it is already moderated.
        let _ = profile_admin_api::post_moderate_profile_string(
            api.api(),
            api_client::models::PostModerateProfileString {
                content_type,
                id: moderation.id.clone(),
                value: moderation.value.clone(),
                accept: r.accept,
                rejected_category: None,
                rejected_details: Some(r.rejected_details.and_then(|v| {
                    if v.is_empty() {
                        None
                    } else {
                        Some(Box::new(ProfileStringModerationRejectedReasonDetails::new(
                            v,
                        )))
                    }
                })),
                move_to_human: if r.move_to_human {
                    Some(Some(true))
                } else {
                    None
                },
            },
        )
        .await;

        Ok(())
    }

    async fn llm_profile_string_moderation(
        profile_string: &str,
        llm: LlmConfigAndClient,
        content_type: ProfileStringModerationContentType,
    ) -> Result<Option<ModerationResult>, TestError> {
        let config = &llm.config;
        let expected_response_lowercase = config.expected_response.to_lowercase();
        let profile_text_paragraph = profile_string.lines().collect::<Vec<&str>>().join(" ");
        let user_text = config.user_text_template.replace(
            ProfileStringModerationConfig::TEMPLATE_PLACEHOLDER_TEXT,
            &profile_text_paragraph,
        );

        // Hide warning about max_tokens as Ollama does not yet
        // support max_completion_tokens.
        #[allow(deprecated)]
        let r = llm
            .client
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
                    return Err(TestError::LlmError).attach_printable(format!(
                        "LLM {content_type} moderation error: no response content"
                    ));
                }
            },
            Ok(None) => {
                return Err(TestError::LlmError)
                    .attach_printable(format!("LLM {content_type} moderation error: no response"));
            }
            Err(e) => {
                return Err(TestError::LlmError)
                    .attach_printable(format!("LLM {content_type} moderation failed: {e}"));
            }
        };

        let response_lowercase = response.trim().to_lowercase();
        let response_first_line = response_lowercase.lines().next().unwrap_or_default();
        let accepted = response_lowercase.starts_with(&expected_response_lowercase)
            || response_first_line.contains(&expected_response_lowercase);
        if config.debug_log_results {
            info!("LLM {content_type} moderation result: '{}'", response);
        }

        let move_to_human = !accepted && config.move_rejected_to_human_moderation;

        let rejected_details = if (!accepted
            && config.add_llm_output_to_user_visible_rejection_details)
            || move_to_human
        {
            Some(response)
        } else {
            None
        };

        Ok(Some(ModerationResult {
            accept: accepted,
            rejected_details,
            move_to_human,
            delete: false,
        }))
    }

    async fn llm_profile_string_moderation_and_retry(
        profile_string: &str,
        llm: LlmConfigAndClient,
        content_type: ProfileStringModerationContentType,
    ) -> Result<Option<ModerationResult>, TestError> {
        let retry_wait_times = &llm.config.retry_wait_times_in_seconds;
        let mut attempt = 0;

        loop {
            match Self::llm_profile_string_moderation(profile_string, llm.clone(), content_type)
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt < retry_wait_times.len() {
                        let wait_time = retry_wait_times[attempt];
                        info!(
                            "LLM {content_type} moderation attempt {} failed, retrying in {} seconds...",
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

impl AdminBotProfileStringModerationLogic {
    pub async fn run_profile_string_moderation(
        content_type: ProfileStringModerationContentType,
        api: &ApiClient,
        config: &ProfileStringModerationConfig,
        moderation_state: &mut ProfileStringModerationState,
    ) -> Result<(), TestError> {
        let logic = Self { content_type };
        loop {
            if let Some(EmptyPage) = logic
                .moderate_one_page(api, config, moderation_state)
                .await?
            {
                break;
            }
        }

        Ok(())
    }
}
