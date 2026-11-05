use api_client::apis::common_api::post_automatic_profile_search_reset_profile_paging;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_automatic_profile_search_reset_profile_paging(&no_token_api).await;
    assert_failure(result)
}

/// Automatic profile search paging can be reset.
#[server_test]
async fn automatic_profile_search_paging_can_be_reset(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let first = post_automatic_profile_search_reset_profile_paging(&account.api()).await?;
    let second = post_automatic_profile_search_reset_profile_paging(&account.api()).await?;

    // Session ids are sequential when previous session was not used.
    assert(first.id + 1 == second.id || first.id == second.id)
}
