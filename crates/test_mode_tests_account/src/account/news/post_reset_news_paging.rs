use api_client::apis::account_api::post_reset_news_paging;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_reset_news_paging(&no_token_api).await;
    assert_failure(result)
}

/// News paging can be reset.
#[server_test]
async fn news_paging_can_be_reset(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = post_reset_news_paging(&account.api()).await?;
    assert_eq(0, result.c.c)?;

    Ok(())
}
