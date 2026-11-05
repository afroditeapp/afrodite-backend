use api_client::apis::account_admin_api::post_create_news_item;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_create_news_item(&no_token_api).await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = post_create_news_item(&account.api()).await;
    assert_failure(result)
}

/// News item can be created by admin.
#[server_test]
async fn news_item_can_be_created(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;

    let _news_id = post_create_news_item(&admin.api()).await?;

    Ok(())
}
