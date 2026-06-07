use api_client::{
    apis::{media_admin_api, media_api},
    models::{AccountId, ContentId, EditVerificationSecurityContent},
};
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
    CreateChatCompletionRequest, ImageUrl,
};
use base64::display::Base64Display;
use config::bot_config_file::internal::{
    AcceptOrReject, AccountVerificationConfigInternal, SecurityContentVerificationConfigInternal,
};
use error_stack::{Result, ResultExt};
use test_mode_utils::{
    AccountVerificationErrorFlags,
    client::{ApiClient, TestError},
};
use tracing::info;

use super::{super::ModerationResult, VerificationMethodAction};
use crate::actions::admin::account_verification::{AccountVerificationState, LlmConfigAndClient};

pub async fn handle_security_content_verification(
    api: &ApiClient,
    config: &AccountVerificationConfigInternal,
    state: &AccountVerificationState,
    account_id: &AccountId,
    method_action: &VerificationMethodAction,
) -> Result<
    (
        Option<EditVerificationSecurityContent>,
        AccountVerificationErrorFlags,
    ),
    TestError,
> {
    let Some(config) = &config.security_content else {
        return Ok((
            None,
            AccountVerificationErrorFlags::SECURITY_CONTENT_VERIFICATION_FAILED,
        ));
    };

    let Some(security_content) =
        media_admin_api::get_security_content_admin_info(&api.api(), &account_id.aid)
            .await
            .change_context(TestError::ApiRequest)?
            .content
            .flatten()
    else {
        return Ok((
            None,
            AccountVerificationErrorFlags::SECURITY_CONTENT_VERIFICATION_FAILED,
        ));
    };

    let (accepted, flags) = match method_action {
        VerificationMethodAction::Accept => (true, AccountVerificationErrorFlags::empty()),
        VerificationMethodAction::Reject
        | VerificationMethodAction::_PersonIdentificationData {
            jpeg_image: None, ..
        } => (false, AccountVerificationErrorFlags::empty()),
        VerificationMethodAction::_PersonIdentificationData {
            jpeg_image: Some(jpeg_image),
            ..
        } => {
            let accepted = handle_check_image_method(
                api,
                config,
                state,
                account_id,
                &security_content.cid,
                jpeg_image,
            )
            .await?;
            if accepted {
                (true, AccountVerificationErrorFlags::empty())
            } else {
                (
                    false,
                    AccountVerificationErrorFlags::SECURITY_CONTENT_VERIFICATION_MISMATCH,
                )
            }
        }
    };

    Ok((
        Some(EditVerificationSecurityContent {
            security_content: security_content.cid,
            verified_value: Some(Some(accepted)),
        }),
        flags,
    ))
}

async fn handle_check_image_method(
    api: &ApiClient,
    config: &SecurityContentVerificationConfigInternal,
    state: &AccountVerificationState,
    aid: &AccountId,
    cid: &ContentId,
    verification_image: &[u8],
) -> Result<bool, TestError> {
    let security_content = media_api::get_content(&api.api(), &aid.aid, &cid.cid, Some(false))
        .await
        .change_context(TestError::ApiRequest)?
        .bytes()
        .await
        .change_context(TestError::ApiRequest)?
        .to_vec();

    let result = if let Some(llm) = state.llm.clone() {
        llm_security_content_verification_and_retry(&security_content, verification_image, llm)
            .await?
    } else {
        None
    };

    Ok(result
        .map(|r| r.accept)
        .unwrap_or_else(|| match config.default_action {
            AcceptOrReject::Accept => true,
            AcceptOrReject::Reject => false,
        }))
}

async fn llm_security_content_verification_and_retry(
    security_content_data: &[u8],
    verification_image_data: &[u8],
    llm: LlmConfigAndClient,
) -> Result<Option<ModerationResult>, TestError> {
    let retry_wait_times = &llm.config.llm.retry_wait_times_in_seconds;
    let mut attempt = 0;

    loop {
        match llm_security_content_verification(
            security_content_data,
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
                    tokio::time::sleep(tokio::time::Duration::from_secs(wait_time as u64)).await;
                    attempt += 1;
                } else {
                    return Err(e);
                }
            }
        }
    }
}

async fn llm_security_content_verification(
    security_content_data: &[u8],
    verification_image_data: &[u8],
    llm: LlmConfigAndClient,
) -> Result<Option<ModerationResult>, TestError> {
    let config = &llm.config;
    let expected_response_lowercase = config.expected_response.to_lowercase();

    let security_content = ChatCompletionRequestMessageContentPartImage {
        image_url: ImageUrl {
            url: format!(
                "data:image/jpeg;base64,{}",
                Base64Display::new(
                    security_content_data,
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
            security_content.into(),
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
            model: config.llm.model.clone(),
            temperature: config.llm.temperature,
            seed: config.llm.seed,
            max_completion_tokens: Some(config.llm.max_tokens),
            max_tokens: Some(config.llm.max_tokens),
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
    if config.llm.debug_log_results {
        info!("LLM security content verification result: '{}'", response);
    }

    Ok(Some(ModerationResult {
        accept: accepted,
        rejected_details: None,
        move_to_human: false,
        delete: false,
    }))
}
