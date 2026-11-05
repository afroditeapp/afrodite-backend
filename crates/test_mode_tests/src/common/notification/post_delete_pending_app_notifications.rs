use api_client::apis::common_api::post_delete_pending_app_notifications;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_delete_pending_app_notifications(&no_token_api, vec![]).await;
    assert_failure(result)
}

/// Deleting no notifications is successful.
#[server_test]
async fn deleting_no_notifications_is_ok(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    post_delete_pending_app_notifications(&account.api(), vec![]).await?;

    Ok(())
}
