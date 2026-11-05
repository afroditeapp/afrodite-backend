use api_client::apis::account_admin_api::delete_news_item;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = delete_news_item(&no_token_api, 999_999).await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = delete_news_item(&account.api(), 999_999).await;
    assert_failure(result)
}

/// Deleting a non existing news item fails because the news translations
/// of the item must be readable before deletion.
#[server_test]
async fn deleting_non_existing_news_item_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;

    let result = delete_news_item(&admin.api(), 999_999).await;
    assert_failure(result)?;

    Ok(())
}
