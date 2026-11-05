use api_client::apis::account_admin_api::post_set_news_publicity;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_set_news_publicity(
        &no_token_api,
        999_999,
        api_client::models::BooleanSetting::new(true),
    )
    .await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = post_set_news_publicity(
        &account.api(),
        999_999,
        api_client::models::BooleanSetting::new(true),
    )
    .await;
    assert_failure(result)
}

/// Publicity of a non existing news item cannot be changed.
#[server_test]
async fn setting_publicity_of_non_existing_news_item_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;

    let result = post_set_news_publicity(
        &admin.api(),
        999_999,
        api_client::models::BooleanSetting::new(true),
    )
    .await;
    assert_failure(result)?;

    Ok(())
}
