use api_client::{apis::account_api::put_sign_in_with_apple, models::PutSignInWithApple};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = put_sign_in_with_apple(&no_token_api, PutSignInWithApple::new()).await;
    assert_failure(result)
}

/// Disassociating Apple sign in is successful when nothing is associated.
/// Associating would require a valid Apple token which cannot be done in
/// the test environment.
#[server_test]
async fn disassociate_is_successful(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    put_sign_in_with_apple(&account.api(), PutSignInWithApple::new()).await?;

    let info = api_client::apis::account_api::get_sign_in_with_info(&account.api()).await?;
    assert(!info.apple)?;

    Ok(())
}
