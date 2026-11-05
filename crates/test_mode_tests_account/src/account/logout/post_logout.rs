use api_client::apis::account_api::post_logout;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_logout(&no_token_api).await;
    assert_failure(result)
}

/// Logging out is successful.
#[server_test]
async fn logout_is_successful(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    post_logout(&account.api()).await?;

    Ok(())
}
