use api_client::apis::account_api::post_get_unread_news_count;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_get_unread_news_count(&no_token_api).await;
    assert_failure(result)
}

/// Unread news count can be read.
#[server_test]
async fn unread_news_count_can_be_read(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let _result = post_get_unread_news_count(&account.api()).await?;

    Ok(())
}

/// Unread news count is per account.
#[server_test]
async fn unread_news_count_is_per_account(mut context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    let account2 = context.new_account().await?;

    let result1 = post_get_unread_news_count(&account1.api()).await?;
    let result2 = post_get_unread_news_count(&account2.api()).await?;
    assert_eq(result1.c.c, result2.c.c)?;

    Ok(())
}
