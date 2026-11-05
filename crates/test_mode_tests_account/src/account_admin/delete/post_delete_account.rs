use api_client::apis::account_admin_api::post_delete_account;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_delete_account(&no_token_api, &admin.account_id_string()).await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    let account2 = context.new_account().await?;

    let result = post_delete_account(&account1.api(), &account2.account_id_string()).await;
    assert_failure(result)?;
    // Also deleting own account requires admin permission.
    let result = post_delete_account(&account1.api(), &account1.account_id_string()).await;
    assert_failure(result)?;

    Ok(())
}

/// Account can be deleted by admin.
#[server_test]
async fn account_can_be_deleted(context: TestContext) -> TestResult {
    let mut context = context;
    let account = context.new_account().await?;
    let admin = context.new_admin().await?;

    post_delete_account(&admin.api(), &account.account_id_string()).await?;

    Ok(())
}
