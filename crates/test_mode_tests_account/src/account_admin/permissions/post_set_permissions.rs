use api_client::{apis::account_admin_api::post_set_permissions, models::Permissions};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_set_permissions(
        &no_token_api,
        &admin.account_id_string(),
        Permissions::default(),
    )
    .await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    let account2 = context.new_account().await?;

    let result = post_set_permissions(
        &account1.api(),
        &account2.account_id_string(),
        Permissions::default(),
    )
    .await;
    assert_failure(result)?;
    let result = post_set_permissions(
        &account1.api(),
        &account1.account_id_string(),
        Permissions::default(),
    )
    .await;
    assert_failure(result)?;

    Ok(())
}

/// Empty permissions cannot be set for an account because permissions
/// cannot be revoked from an admin by an admin.
#[server_test]
async fn settings_permissions_of_normal_account_is_allowed(context: TestContext) -> TestResult {
    let mut context = context;
    let account = context.new_account().await?;
    let admin = context.new_admin().await?;

    // Setting same permissions than the account already has is allowed.
    post_set_permissions(
        &admin.api(),
        &account.account_id_string(),
        Permissions::default(),
    )
    .await?;

    Ok(())
}
