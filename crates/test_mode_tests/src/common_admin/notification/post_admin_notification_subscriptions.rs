use api_client::{
    apis::common_admin_api::{
        get_admin_notification_subscriptions, post_admin_notification_subscriptions,
    },
    models::AdminNotification,
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result =
        post_admin_notification_subscriptions(&no_token_api, AdminNotification::default()).await;
    assert_failure(result)
}

/// Request without admin permission fails. A normal account has no
/// permissions enabled.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result =
        post_admin_notification_subscriptions(&account.api(), AdminNotification::default()).await;
    assert_failure(result)
}

/// Admin notification subscriptions can be saved by admin account.
#[server_test]
async fn subscriptions_can_be_saved_by_admin(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;

    let new_subscriptions = api_client::models::AdminNotification {
        process_reports: Some(true),
        ..Default::default()
    };
    post_admin_notification_subscriptions(&admin.api(), new_subscriptions).await?;

    let subscriptions = get_admin_notification_subscriptions(&admin.api()).await?;
    assert_eq(Some(true), subscriptions.process_reports)?;

    Ok(())
}
