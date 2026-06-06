use std::{fmt::Debug, sync::Arc};

use api_client::{
    apis::{media_admin_api, media_api},
    models::{PostMediaContentFaceVerifiedValue, PostMediaContentFaceVerifiedValueItem},
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
    AcceptOrReject, FaceVerificationConfig, LlmFaceVerificationConfig,
};
use error_stack::{Result, ResultExt};
use futures::{StreamExt, stream};
use test_mode_utils::client::{ApiClient, TestError};
use tracing::info;

use super::{EmptyPage, ModerationResult};

#[derive(Debug, Clone)]
struct LlmConfigAndClient {
    config: Arc<LlmFaceVerificationConfig>,
    client: Client<OpenAIConfig>,
}

#[derive(Debug)]
pub struct FaceVerificationState {
    llm: Option<LlmConfigAndClient>,
}

impl FaceVerificationState {
    pub fn new(config: &FaceVerificationConfig, reqwest_client: reqwest::Client) -> Self {
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
pub struct AdminBotFaceVerificationLogic;

impl AdminBotFaceVerificationLogic {
    async fn verify_one_page(
        api: &ApiClient,
        config: &FaceVerificationConfig,
        state: &mut FaceVerificationState,
    ) -> Result<Option<EmptyPage>, TestError> {
        let list = media_admin_api::get_media_content_face_verified_null_list(&api.api())
            .await
            .change_context(TestError::ApiRequest)?;

        if list.values.is_empty() {
            return Ok(Some(EmptyPage));
        }

        let mut stream = stream::iter(list.values)
            .map(|account_values| {
                let api = api.clone();
                let llm = state.llm.clone();
                async move { Self::verify_for_account(&api, config, llm, account_values).await }
            })
            .buffer_unordered(config.concurrency.into());

        loop {
            match stream.next().await {
                Some(Ok(())) => (),
                Some(Err(e)) => return Err(e),
                None => return Ok(None),
            }
        }
    }

    async fn verify_for_account(
        api: &ApiClient,
        config: &FaceVerificationConfig,
        llm: Option<LlmConfigAndClient>,
        values_by_account: api_client::models::MediaContentFaceVerifiedNullByAccount,
    ) -> Result<(), TestError> {
        let security_image = media_api::get_content(
            &api.api(),
            &values_by_account.account_id.aid,
            &values_by_account.security_content.cid,
            Some(false),
        )
        .await
        .change_context(TestError::ApiRequest)?
        .bytes()
        .await
        .change_context(TestError::ApiRequest)?
        .to_vec();

        let mut values = Vec::with_capacity(values_by_account.values.len());

        for content_id in values_by_account.values {
            let content_image = media_api::get_content(
                &api.api(),
                &values_by_account.account_id.aid,
                &content_id.cid,
                Some(false),
            )
            .await
            .change_context(TestError::ApiRequest)?
            .bytes()
            .await
            .change_context(TestError::ApiRequest)?
            .to_vec();

            let result = if let Some(llm) = llm.clone() {
                Self::llm_face_verification_and_retry(&security_image, &content_image, llm).await?
            } else {
                None
            };

            let value =
                result
                    .map(|r| Some(r.accept))
                    .unwrap_or_else(|| match config.default_action {
                        AcceptOrReject::Accept => Some(true),
                        AcceptOrReject::Reject => Some(false),
                    });

            if value.is_some() {
                values.push(PostMediaContentFaceVerifiedValueItem {
                    content_id: Box::new(content_id),
                    value: Some(value),
                });
            }
        }

        if !values.is_empty() {
            media_admin_api::post_media_content_face_verified_value(
                &api.api(),
                PostMediaContentFaceVerifiedValue {
                    account_id: values_by_account.account_id,
                    security_content: values_by_account.security_content.clone(),
                    values,
                },
            )
            .await
            .change_context(TestError::ApiRequest)?;
        }

        Ok(())
    }

    async fn llm_face_verification(
        security_image_data: &[u8],
        content_image_data: &[u8],
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

        let content_image = ChatCompletionRequestMessageContentPartImage {
            image_url: ImageUrl {
                url: format!(
                    "data:image/jpeg;base64,{}",
                    Base64Display::new(
                        content_image_data,
                        &base64::engine::general_purpose::STANDARD
                    ),
                ),
                detail: None,
            },
        };

        let user_message = ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Array(vec![
                security_image.into(),
                content_image.into(),
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
                    return Err(TestError::LlmError)
                        .attach_printable("LLM face verification error: no response content");
                }
            },
            Ok(None) => {
                return Err(TestError::LlmError)
                    .attach_printable("LLM face verification error: no response");
            }
            Err(e) => {
                return Err(TestError::LlmError)
                    .attach_printable(format!("LLM face verification failed: {e}"));
            }
        };

        let response_lowercase = response.trim().to_lowercase();
        let response_first_line = response_lowercase.lines().next().unwrap_or_default();
        let accepted = response_lowercase.starts_with(&expected_response_lowercase)
            || response_first_line.contains(&expected_response_lowercase);
        if config.debug_log_results {
            info!("LLM face verification result: '{}'", response);
        }

        Ok(Some(ModerationResult {
            accept: accepted,
            rejected_details: None,
            move_to_human: false,
            delete: false,
        }))
    }

    async fn llm_face_verification_and_retry(
        security_image_data: &[u8],
        content_image_data: &[u8],
        llm: LlmConfigAndClient,
    ) -> Result<Option<ModerationResult>, TestError> {
        let retry_wait_times = &llm.config.retry_wait_times_in_seconds;
        let mut attempt = 0;

        loop {
            match Self::llm_face_verification(security_image_data, content_image_data, llm.clone())
                .await
            {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt < retry_wait_times.len() {
                        let wait_time = retry_wait_times[attempt];
                        info!(
                            "LLM face verification attempt {} failed, retrying in {} seconds...",
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

    pub async fn run_face_verification(
        api: &ApiClient,
        config: &FaceVerificationConfig,
        state: &mut FaceVerificationState,
    ) -> Result<(), TestError> {
        loop {
            if let Some(EmptyPage) = Self::verify_one_page(api, config, state).await? {
                break;
            }
        }

        Ok(())
    }
}
