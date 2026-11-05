use api_client::{
    apis::common_api::{
        post_automatic_profile_search_get_next_profile_page,
        post_automatic_profile_search_reset_profile_paging,
    },
    models::AutomaticProfileSearchIteratorSessionId,
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_automatic_profile_search_get_next_profile_page(
        &no_token_api,
        AutomaticProfileSearchIteratorSessionId::new(0),
    )
    .await;
    assert_failure(result)
}

/// Invalid iterator session id does not cause a server error.
/// The page is empty when automatic profile search has not
/// happened yet and error flag is set when it has.
#[server_test]
async fn invalid_iterator_session_id_does_not_cause_server_error(
    mut context: TestContext,
) -> TestResult {
    let account = context.new_account().await?;

    let page = post_automatic_profile_search_get_next_profile_page(
        &account.api(),
        AutomaticProfileSearchIteratorSessionId::new(0),
    )
    .await?;
    assert(!page.error.unwrap_or(false))
}

/// Next automatic profile search page can be fetched after reset.
#[server_test]
async fn next_profile_page_can_be_fetched(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let iterator_session_id =
        post_automatic_profile_search_reset_profile_paging(&account.api()).await?;

    let page =
        post_automatic_profile_search_get_next_profile_page(&account.api(), iterator_session_id)
            .await?;
    assert(!page.error.unwrap_or(false))?;
    assert(!page.error_invalid_iterator_session_id.unwrap_or(false))
}
