use api_client::apis::account_api::get_account_state;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_account_state(&no_token_api).await;
    assert_failure(result)
}

/// Account state can be read by the account itself.
#[server_test]
async fn own_state_can_be_read(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let state = get_account_state(&account.api()).await?;
    assert_eq(
        api_client::models::ProfileVisibility::Private,
        state.visibility,
    )?;

    Ok(())
}
