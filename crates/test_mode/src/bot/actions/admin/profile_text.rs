use std::{fmt::Debug, time::Instant};

use api_client::{apis::profile_admin_api, models::ProfileTextModerationRejectedReasonDetails};
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequest},
    Client,
};
use async_trait::async_trait;
use config::{bot_config_file::ProfileTextModerationConfig, Config};
use error_stack::{Result, ResultExt};
use tracing::error;
use unicode_segmentation::UnicodeSegmentation;

use super::{BotAction, BotState, EmptyPage};
use crate::client::{ApiClient, TestError};

#[derive(Debug)]
pub struct ProfileTextModerationState {
    moderation_started: Option<Instant>,
    client: Client<OpenAIConfig>,
}

#[derive(Debug)]
pub struct AdminBotProfileTextModerationLogic;

impl AdminBotProfileTextModerationLogic {
    async fn moderate_one_page(
        api: &ApiClient,
        config: &ProfileTextModerationConfig,
        client: &Client<OpenAIConfig>,
        server_config: &Config,
    ) -> Result<Option<EmptyPage>, TestError> {
        let list = profile_admin_api::get_profile_text_pending_moderation_list(api.profile(), true)
            .await
            .change_context(TestError::ApiRequest)?;

        if list.values.is_empty() {
            return Ok(Some(EmptyPage));
        }

        let expected_response_lowercase = config.expected_response.to_lowercase();

        for request in list.values {
            // Allow texts with only single visible character
            if config.accept_single_visible_character && request.text.graphemes(true).count() == 1 {
                // Ignore errors as the user might have changed the text to
                // another one or it is already moderated.
                let _ = profile_admin_api::post_moderate_profile_text(
                    api.profile(),
                    api_client::models::PostModerateProfileText {
                        id: request.id.clone(),
                        text: request.text.clone(),
                        accept: true,
                        rejected_category: None,
                        rejected_details: None,
                        move_to_human: None,
                    },
                )
                .await;

                continue;
            }

            let profile_text_paragraph = request.text.lines().collect::<Vec<&str>>().join(" ");

            let user_text = config.user_text_template.replace(
                ProfileTextModerationConfig::TEMPLATE_FORMAT_ARGUMENT,
                &profile_text_paragraph,
            );

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
                    max_tokens: Some(10_000),
                    ..Default::default()
                })
                .await;
            let response = match r.map(|r| r.choices.into_iter().next()) {
                Ok(Some(r)) => match r.message.content {
                    Some(response) => response,
                    None => {
                        error!("Profile text moderation error: no response content from LLM");
                        return Ok(Some(EmptyPage));
                    }
                },
                Ok(None) => {
                    error!("Profile text moderation error: no response from LLM");
                    return Ok(Some(EmptyPage));
                }
                Err(e) => {
                    error!("Profile text moderation error: {}", e);
                    return Ok(Some(EmptyPage));
                }
            };

            let response_lowercase = response.trim().to_lowercase();
            let response_first_line = response_lowercase.lines().next().unwrap_or_default();
            let accepted = response_lowercase.starts_with(&expected_response_lowercase)
                || response_first_line.contains(&expected_response_lowercase);
            let rejected_details = if !accepted && server_config.debug_mode() {
                Some(Some(Box::new(
                    ProfileTextModerationRejectedReasonDetails::new(response),
                )))
            } else {
                None
            };

            let move_to_human = if !accepted && config.move_rejected_to_human_moderation {
                Some(Some(true))
            } else {
                None
            };

            // Ignore errors as the user might have changed the text to
            // another one or it is already moderated.
            let _ = profile_admin_api::post_moderate_profile_text(
                api.profile(),
                api_client::models::PostModerateProfileText {
                    id: request.id.clone(),
                    text: request.text.clone(),
                    accept: accepted,
                    rejected_category: None,
                    rejected_details,
                    move_to_human,
                },
            )
            .await;
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

        let moderation_state =
            state
                .admin
                .profile_text
                .get_or_insert_with(|| ProfileTextModerationState {
                    moderation_started: None,
                    client: Client::with_config(
                        OpenAIConfig::new()
                            .with_api_base(config.openai_api_url.to_string())
                            .with_api_key(""),
                    ),
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
            if let Some(EmptyPage) = Self::moderate_one_page(
                &state.api,
                config,
                &moderation_state.client,
                &state.server_config,
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

        Ok(())
    }
}
