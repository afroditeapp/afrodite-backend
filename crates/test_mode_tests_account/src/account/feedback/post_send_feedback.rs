use api_client::{apis::account_api::post_send_feedback, models::SendFeedback};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_send_feedback(
        &no_token_api,
        SendFeedback::new("Hello".to_string(), "general".to_string()),
    )
    .await;
    assert_failure(result)
}

/// Sending fails when the feedback feature is disabled.
#[server_test]
async fn disabled_feedback_returns_unknown_category_error(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = post_send_feedback(
        &account.api(),
        SendFeedback::new("Hello".to_string(), "general".to_string()),
    )
    .await?;
    assert(result.error_unknown_category.unwrap_or(false))
}
