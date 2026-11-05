use api_client::apis::common_api::get_push_notification_info;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_push_notification_info(&no_token_api).await;
    assert_failure(result)
}

/// Device token is not set by default.
#[server_test]
async fn device_token_is_not_set_by_default(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let info = get_push_notification_info(&account.api()).await?;
    assert(info.device_token.is_none())
}
