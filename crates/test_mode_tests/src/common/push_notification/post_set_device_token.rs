use api_client::{
    apis::common_api::{get_push_notification_info, post_set_device_token},
    models::PushNotificationDeviceToken,
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_set_device_token(
        &no_token_api,
        PushNotificationDeviceToken::new("token".to_string()),
    )
    .await;
    assert_failure(result)
}

/// Device token can be set and read.
#[server_test]
async fn device_token_can_be_set(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let encryption_key = post_set_device_token(
        &account.api(),
        PushNotificationDeviceToken::new("token".to_string()),
    )
    .await?;
    assert(!encryption_key.key.is_empty())?;

    let info = get_push_notification_info(&account.api()).await?;
    let device_token = info.device_token.ok_or(TestError::MissingValue.report())?;
    assert_eq("token".to_string(), device_token.token)
}
