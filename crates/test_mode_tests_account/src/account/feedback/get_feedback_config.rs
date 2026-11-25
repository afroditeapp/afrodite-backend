use api_client::apis::account_api::get_feedback_config;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_feedback_config(&no_token_api).await;
    assert_failure(result)
}

/// Feedback feature is disabled by default and the config is empty.
#[server_test]
async fn disabled_feedback_returns_empty_config(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let config = get_feedback_config(&account.api()).await?;
    assert_eq(0, config.body_max_grapheme_cluster_count)?;
    assert_eq(0, config.categories.unwrap_or_default().len())
}
