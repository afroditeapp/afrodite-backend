use api_client::{
    apis::common_admin_api::{get_admin_notification_settings, post_admin_notification_settings},
    models::AdminNotificationSettings,
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result =
        post_admin_notification_settings(&no_token_api, AdminNotificationSettings::default()).await;
    assert_failure(result)
}

/// Request without admin permission fails. A normal account has no
/// permissions enabled.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result =
        post_admin_notification_settings(&account.api(), AdminNotificationSettings::default())
            .await;
    assert_failure(result)
}

/// Admin notification settings can be saved by admin account.
#[server_test]
async fn settings_can_be_saved_by_admin(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;

    let new_settings = AdminNotificationSettings::new(0, 3600, 0x1 | 0x2);
    post_admin_notification_settings(&admin.api(), new_settings.clone()).await?;

    let settings = get_admin_notification_settings(&admin.api()).await?;
    assert_eq(new_settings.daily_enabled_time_start_seconds, 3600)?;
    assert_eq(new_settings, settings)?;

    Ok(())
}
