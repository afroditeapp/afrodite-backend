use std::{fmt::Debug, sync::Arc};

use api_client::{
    apis::{media_admin_api, media_api},
    models::{
        AccountId, ContentId, PostSecurityContentVerificationQueueRemoveNextItem,
        PostSecurityContentVerifiedValue, SecurityContentVerificationQueueAdminItem,
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
use base64::display::Base64Display;
use config::bot_config_file::internal::{
    LlmSecurityContentVerificationConfig, SecurityContentVerificationConfig, VerificationAction,
};
use error_stack::{Result, ResultExt};
use test_mode_utils::client::{ApiClient, TestError};
use tracing::info;

use super::{EmptyPage, ModerationResult};

#[derive(Debug, Clone)]
struct LlmConfigAndClient {
    config: Arc<LlmSecurityContentVerificationConfig>,
    client: Client<OpenAIConfig>,
}

#[derive(Debug, Default)]
pub struct SecurityContentVerificationState {
    llm: Option<LlmConfigAndClient>,
}

impl SecurityContentVerificationState {
    pub fn new(
        config: &SecurityContentVerificationConfig,
        reqwest_client: reqwest::Client,
    ) -> Self {
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
pub struct AdminBotSecurityContentVerificationLogic;

enum VerificationMethodAction {
    Accept,
    Reject,
    _CheckImage(Vec<u8>),
}

impl AdminBotSecurityContentVerificationLogic {
    async fn verify_one_page(
        api: &ApiClient,
        config: &SecurityContentVerificationConfig,
        state: &mut SecurityContentVerificationState,
    ) -> Result<Option<EmptyPage>, TestError> {
        let item = match Self::get_next_queue_item(api).await? {
            Some(item) => item,
            None => return Ok(Some(EmptyPage)),
        };

        let value = match Self::parse_verification_method_action(
            config,
            &item.verification_method,
            &item.verification_data,
        )? {
            VerificationMethodAction::Accept => Some(true),
            VerificationMethodAction::Reject => Some(false),
            VerificationMethodAction::_CheckImage(verification_image) => {
                Self::handle_check_image_method(
                    api,
                    config,
                    state,
                    &item.account_id,
                    &item.security_content,
                    verification_image,
                )
                .await?
            }
        };

        let account_id = (*item.account_id).clone();
        let security_content = (*item.security_content).clone();

        media_admin_api::post_security_content_verified_value(
            &api.api(),
            PostSecurityContentVerifiedValue {
                account_id: Box::new(account_id.clone()),
                security_content: Box::new(security_content),
                value: Some(value),
            },
        )
        .await
        .change_context(TestError::ApiRequest)?;

        Self::remove_next_queue_item(api, account_id).await?;

        Ok(None)
    }

    async fn llm_security_content_verification(
        security_image_data: &[u8],
        verification_image_data: &[u8],
        llm: LlmConfigAndClient,
    ) -> Result<Option<ModerationResult>, TestError> {
        let config = &llm.config;
        let expected_response_lowercase = config.expected_response.to_lowercase();

        let security_image = ChatCompletionRequestMessageContentPartImage {
            image_url: ImageUrl {
                url: format!(
                    "data:image/jpeg;base64,{}",
                    Base64Display::new(
                        security_image_data,
                        &base64::engine::general_purpose::STANDARD
                    ),
                ),
                detail: None,
            },
        };

        let verification_image = ChatCompletionRequestMessageContentPartImage {
            image_url: ImageUrl {
                url: format!(
                    "data:image/jpeg;base64,{}",
                    Base64Display::new(
                        verification_image_data,
                        &base64::engine::general_purpose::STANDARD
                    ),
                ),
                detail: None,
            },
        };

        let user_message = ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Array(vec![
                security_image.into(),
                verification_image.into(),
            ]),
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
                    ChatCompletionRequestMessage::User(user_message),
                ],
                model: config.model.clone(),
                temperature: config.temperature,
                seed: config.seed,
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
                        "LLM security content verification error: no response content",
                    );
                }
            },
            Ok(None) => {
                return Err(TestError::LlmError)
                    .attach_printable("LLM security content verification error: no response");
            }
            Err(e) => {
                return Err(TestError::LlmError)
                    .attach_printable(format!("LLM security content verification failed: {e}"));
            }
        };

        let response_lowercase = response.trim().to_lowercase();
        let response_first_line = response_lowercase.lines().next().unwrap_or_default();
        let accepted = response_lowercase.starts_with(&expected_response_lowercase)
            || response_first_line.contains(&expected_response_lowercase);
        if config.debug_log_results {
            info!("LLM security content verification result: '{}'", response);
        }

        Ok(Some(ModerationResult {
            accept: accepted,
            rejected_details: None,
            move_to_human: false,
            delete: false,
        }))
    }

    async fn handle_check_image_method(
        api: &ApiClient,
        config: &SecurityContentVerificationConfig,
        state: &SecurityContentVerificationState,
        aid: &AccountId,
        cid: &ContentId,
        verification_image: Vec<u8>,
    ) -> Result<Option<bool>, TestError> {
        let security_image = media_api::get_content(&api.api(), &aid.aid, &cid.cid, Some(false))
            .await
            .change_context(TestError::ApiRequest)?
            .bytes()
            .await
            .change_context(TestError::ApiRequest)?
            .to_vec();

        let result = if let Some(llm) = state.llm.clone() {
            Self::llm_security_content_verification_and_retry(
                &security_image,
                &verification_image,
                llm,
            )
            .await?
        } else {
            None
        };

        Ok(result
            .map(|r| Some(r.accept))
            .unwrap_or_else(|| match config.default_action {
                VerificationAction::Accept => Some(true),
                VerificationAction::Reject => Some(false),
            }))
    }

    async fn llm_security_content_verification_and_retry(
        security_image_data: &[u8],
        verification_image_data: &[u8],
        llm: LlmConfigAndClient,
    ) -> Result<Option<ModerationResult>, TestError> {
        let retry_wait_times = &llm.config.retry_wait_times_in_seconds;
        let mut attempt = 0;

        loop {
            match Self::llm_security_content_verification(
                security_image_data,
                verification_image_data,
                llm.clone(),
            )
            .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt < retry_wait_times.len() {
                        let wait_time = retry_wait_times[attempt];
                        info!(
                            "LLM security content verification attempt {} failed, retrying in {} seconds...",
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

    fn parse_verification_method_action(
        config: &SecurityContentVerificationConfig,
        verification_method: &str,
        _verification_data: &str,
    ) -> Result<VerificationMethodAction, TestError> {
        match verification_method.trim().to_lowercase().as_str() {
            "debug_accept" if config.allowed_methods.debug_accept => {
                Ok(VerificationMethodAction::Accept)
            }
            "debug_reject" if config.allowed_methods.debug_reject => {
                Ok(VerificationMethodAction::Reject)
            }
            // TODO: eudi
            _ => Err(TestError::AdminBotInternalError).attach_printable(
                "Unsupported or disabled security content verification method".to_string(),
            ),
        }
    }

    async fn get_next_queue_item(
        api: &ApiClient,
    ) -> Result<Option<SecurityContentVerificationQueueAdminItem>, TestError> {
        let response =
            media_admin_api::get_security_content_verification_queue_next_item(&api.api())
                .await
                .change_context(TestError::ApiRequest)?
                .item
                .flatten()
                .map(|item| *item);

        Ok(response)
    }

    async fn remove_next_queue_item(
        api: &ApiClient,
        account_id: api_client::models::AccountId,
    ) -> Result<(), TestError> {
        media_admin_api::post_security_content_verification_queue_remove_next_item(
            &api.api(),
            PostSecurityContentVerificationQueueRemoveNextItem {
                account_id: Box::new(account_id),
            },
        )
        .await
        .change_context(TestError::ApiRequest)?;

        Ok(())
    }

    pub async fn run_security_content_verification(
        api: &ApiClient,
        config: &SecurityContentVerificationConfig,
        state: &mut SecurityContentVerificationState,
    ) -> Result<(), TestError> {
        loop {
            if Self::verify_one_page(api, config, state).await?.is_some() {
                return Ok(());
            }
        }
    }
}
