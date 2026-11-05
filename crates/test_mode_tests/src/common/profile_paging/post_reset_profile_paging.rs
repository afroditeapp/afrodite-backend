use api_client::apis::common_api::{post_get_next_profile_page, post_reset_profile_paging};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_reset_profile_paging(&no_token_api).await;
    assert_failure(result)
}

/// Profile paging can be reset.
#[server_test]
async fn profile_paging_can_be_reset(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let first = post_reset_profile_paging(&account.api()).await?;
    let second = post_reset_profile_paging(&account.api()).await?;

    // Second reset creates a new iterator session when the
    // previous session id was used for paging.
    let _ = post_get_next_profile_page(&account.api(), first.clone()).await?;
    let _ = post_get_next_profile_page(&account.api(), second.clone()).await?;

    let third = post_reset_profile_paging(&account.api()).await?;
    assert(second.id != third.id)
}
