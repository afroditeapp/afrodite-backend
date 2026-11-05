use api_client::apis::account_admin_api::post_admin_logout;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_admin_logout(&no_token_api, &admin.account_id_string()).await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    let account2 = context.new_account().await?;

    let result = post_admin_logout(&account1.api(), &account2.account_id_string()).await;
    assert_failure(result)?;
    let result = post_admin_logout(&account1.api(), &account1.account_id_string()).await;
    assert_failure(result)?;

    Ok(())
}

/// Another account can be logged out by admin.
#[server_test]
async fn other_account_can_be_logged_out(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let admin = context.new_admin().await?;

    post_admin_logout(&admin.api(), &account.account_id_string()).await?;

    Ok(())
}
