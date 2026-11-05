use api_client::{
    apis::account_api::post_get_news_page,
    models::{NewsIteratorState, PublicationId},
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_get_news_page(
        &no_token_api,
        "en",
        NewsIteratorState::new(PublicationId::new(0), 0),
    )
    .await;
    assert_failure(result)
}

/// News page can be read.
#[server_test]
async fn news_page_can_be_read(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    post_get_news_page(
        &account.api(),
        "en",
        NewsIteratorState::new(PublicationId::new(0), 0),
    )
    .await?;

    Ok(())
}
