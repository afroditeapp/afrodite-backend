use api_client::apis::account_api::get_account_ban_time;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_account_ban_time(&no_token_api, &account.account_id_string()).await;
    assert_failure(result)
}

/// Own ban time can be read when the account is not banned.
#[server_test]
async fn own_ban_time_can_be_read(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = get_account_ban_time(&account.api(), &account.account_id_string()).await?;
    assert(result.banned_until.is_none())?;

    Ok(())
}

/// Ban time of another account cannot be read without admin permission.
#[server_test]
async fn other_account_ban_time_fails(mut context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    let account2 = context.new_account().await?;

    let result = get_account_ban_time(&account1.api(), &account2.account_id_string()).await;
    assert_failure(result)?;

    Ok(())
}
