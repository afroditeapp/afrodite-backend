use api_client::{apis::account_api::put_setting_unlimited_likes, models::BooleanSetting};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = put_setting_unlimited_likes(&no_token_api, BooleanSetting::default()).await;
    assert_failure(result)
}

/// Unlimited likes setting can be enabled and disabled multiple times.
#[server_test]
async fn unlimited_likes_can_be_changed(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    put_setting_unlimited_likes(&account.api(), BooleanSetting::new(true)).await?;

    put_setting_unlimited_likes(&account.api(), BooleanSetting::new(false)).await?;

    Ok(())
}
