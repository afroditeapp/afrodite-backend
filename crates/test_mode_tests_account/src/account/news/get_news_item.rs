use api_client::apis::account_api::get_news_item;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_news_item(&no_token_api, 1, "en", None).await;
    assert_failure(result)
}

/// Non existing news item returns a record not found error.
#[server_test]
async fn non_existing_news_item_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = get_news_item(&account.api(), 999_999, "en", None).await;
    assert_failure(result)?;

    Ok(())
}

/// News items are global, so the same item id behaves the same for all
/// accounts.
#[server_test]
async fn news_item_is_not_account_specific(mut context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    let _account2 = context.new_account().await?;

    let result1 = get_news_item(&account1.api(), 999_999, "en", None).await;
    assert_failure(result1)?;

    Ok(())
}
