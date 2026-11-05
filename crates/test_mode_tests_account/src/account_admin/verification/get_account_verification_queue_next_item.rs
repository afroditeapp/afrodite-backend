use api_client::apis::account_admin_api::get_account_verification_queue_next_item;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_account_verification_queue_next_item(&no_token_api).await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = get_account_verification_queue_next_item(&account.api()).await;
    assert_failure(result)
}

/// Verification queue is empty by default.
#[server_test]
async fn queue_is_empty_by_default(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;

    let result = get_account_verification_queue_next_item(&admin.api()).await?;
    assert(result.item.is_none())?;

    Ok(())
}
