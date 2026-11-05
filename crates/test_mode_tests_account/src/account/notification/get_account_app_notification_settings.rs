use api_client::apis::account_api::get_account_app_notification_settings;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_account_app_notification_settings(&no_token_api).await;
    assert_failure(result)
}

/// Notification settings can be read. New notifications are enabled by
/// default.
#[server_test]
async fn settings_can_be_read(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let settings = get_account_app_notification_settings(&account.api()).await?;
    assert(settings.news)?;

    Ok(())
}

/// Notification settings are per account.
#[server_test]
async fn settings_are_per_account(mut context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    let account2 = context.new_account().await?;

    api_client::apis::account_api::post_account_app_notification_settings(
        &account1.api(),
        api_client::models::AccountAppNotificationSettings::new(false),
    )
    .await?;

    let settings1 = get_account_app_notification_settings(&account1.api()).await?;
    assert(!settings1.news)?;
    let settings2 = get_account_app_notification_settings(&account2.api()).await?;
    assert(settings2.news)?;

    Ok(())
}
