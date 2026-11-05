use api_client::apis::common_admin_api::get_admin_notification_settings;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_admin_notification_settings(&no_token_api).await;
    assert_failure(result)
}

/// Request without admin permission fails. A normal account has no
/// permissions enabled.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = get_admin_notification_settings(&account.api()).await;
    assert_failure(result)
}

/// Admin notification settings can be read by admin account.
/// Defaults allow notifications for the whole day on all weekdays.
#[server_test]
async fn settings_can_be_read_by_admin(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;

    let settings = get_admin_notification_settings(&admin.api()).await?;
    assert_eq(0, settings.daily_enabled_time_start_seconds)?;
    assert_eq(86_399, settings.daily_enabled_time_end_seconds)?;
    assert_eq(127, settings.weekdays)?;

    Ok(())
}
