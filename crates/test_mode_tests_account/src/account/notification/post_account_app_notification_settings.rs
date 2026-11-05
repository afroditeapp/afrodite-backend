use api_client::apis::account_api::post_account_app_notification_settings;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_account_app_notification_settings(
        &no_token_api,
        api_client::models::AccountAppNotificationSettings::new(true),
    )
    .await;
    assert_failure(result)
}

/// Notification settings can be updated.
#[server_test]
async fn settings_can_be_updated(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    post_account_app_notification_settings(
        &account.api(),
        api_client::models::AccountAppNotificationSettings::new(false),
    )
    .await?;

    let settings =
        api_client::apis::account_api::get_account_app_notification_settings(&account.api())
            .await?;
    assert(!settings.news)?;

    Ok(())
}
