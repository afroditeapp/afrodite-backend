use api_client::apis::account_admin_api::get_all_admins;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_all_admins(&no_token_api).await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = get_all_admins(&account.api()).await;
    assert_failure(result)
}

/// Admin list can be read by admin.
#[server_test]
async fn all_admins_can_be_read(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;

    let admins = get_all_admins(&admin.api()).await?;
    assert_eq(1, admins.admins.len())?;

    Ok(())
}
