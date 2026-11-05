use api_client::apis::common_api::get_pending_app_notifications;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_pending_app_notifications(&no_token_api).await;
    assert_failure(result)
}

/// There are no pending notifications by default.
#[server_test]
async fn no_pending_notifications_by_default(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let notifications = get_pending_app_notifications(&account.api()).await?;
    assert(notifications.is_empty())
}
