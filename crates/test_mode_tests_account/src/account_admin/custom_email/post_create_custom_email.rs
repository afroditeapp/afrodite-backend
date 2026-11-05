//! Permission check tests for custom email APIs. Successful admin tests
//! are not done because the APIs send emails via SMTP which is not
//! available in the test environment.

use api_client::apis::account_admin_api::post_create_custom_email;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_create_custom_email(&no_token_api).await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = post_create_custom_email(&account.api()).await;
    assert_failure(result)
}

/// Creating a custom email draft is successful and does not send an email.
#[server_test]
async fn custom_email_draft_can_be_created(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;

    let _email_id = post_create_custom_email(&admin.api()).await?;

    Ok(())
}
