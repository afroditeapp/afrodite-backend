use api_client::{
    apis::common_api::{post_get_next_profile_page, post_reset_profile_paging},
    models::ProfileIteratorSessionId,
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_get_next_profile_page(&no_token_api, ProfileIteratorSessionId::new(0)).await;
    assert_failure(result)
}

/// Next profile page can be fetched with a valid iterator session id.
#[server_test]
async fn next_profile_page_can_be_fetched(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let iterator_session_id = post_reset_profile_paging(&account.api()).await?;

    let page = post_get_next_profile_page(&account.api(), iterator_session_id).await?;
    assert(!page.error.unwrap_or(false))?;
    assert(!page.error_invalid_iterator_session_id.unwrap_or(false))
}

/// Invalid iterator session id is rejected with an error page.
#[server_test]
async fn invalid_iterator_session_id_returns_error_page(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let page = post_get_next_profile_page(&account.api(), ProfileIteratorSessionId::new(0)).await?;
    assert(page.error_invalid_iterator_session_id.unwrap_or(false))
}
