use api_client::{
    apis::common_admin_api::get_admin_notification_subscriptions, models::AdminNotification,
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_admin_notification_subscriptions(&no_token_api).await;
    assert_failure(result)
}

/// Request without admin permission fails. A normal account has no
/// permissions enabled.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = get_admin_notification_subscriptions(&account.api()).await;
    assert_failure(result)
}

/// Admin notification subscriptions can be read by admin account.
#[server_test]
async fn subscriptions_can_be_read_by_admin(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;

    let subscriptions = get_admin_notification_subscriptions(&admin.api()).await?;
    assert_eq(AdminNotification::default(), subscriptions)?;

    Ok(())
}
