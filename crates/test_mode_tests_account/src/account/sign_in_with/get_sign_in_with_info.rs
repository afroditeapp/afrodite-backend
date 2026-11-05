use api_client::apis::account_api::get_sign_in_with_info;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_sign_in_with_info(&no_token_api).await;
    assert_failure(result)
}

/// Sign in with state is empty by default.
#[server_test]
async fn sign_in_with_state_is_empty_by_default(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let info = get_sign_in_with_info(&account.api()).await?;
    assert(!info.apple)?;
    assert(!info.google)?;

    Ok(())
}

/// Sign in with state is per account.
#[server_test]
async fn sign_in_with_state_is_per_account(mut context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    let account2 = context.new_account().await?;

    let info1 = get_sign_in_with_info(&account1.api()).await?;
    assert(!info1.apple)?;
    assert(!info1.google)?;

    let info2 = get_sign_in_with_info(&account2.api()).await?;
    assert(!info2.apple)?;
    assert(!info2.google)?;

    Ok(())
}
