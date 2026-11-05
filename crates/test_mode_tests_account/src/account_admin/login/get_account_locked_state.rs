use api_client::apis::account_admin_api::get_account_locked_state;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_account_locked_state(&no_token_api, &admin.account_id_string()).await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    let account2 = context.new_account().await?;

    let result = get_account_locked_state(&account1.api(), &account2.account_id_string()).await;
    assert_failure(result)?;
    let result = get_account_locked_state(&account1.api(), &account1.account_id_string()).await;
    assert_failure(result)?;

    Ok(())
}

/// Locked state of an account can be read by admin. Account is not locked
/// by default.
#[server_test]
async fn locked_state_can_be_read(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let admin = context.new_admin().await?;

    let state = get_account_locked_state(&admin.api(), &account.account_id_string()).await?;
    assert(!state.locked)?;

    Ok(())
}
