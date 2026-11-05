//! Permission check tests for custom email APIs. Successful admin tests
//! are not done because the APIs send emails via SMTP which is not
//! available in the test environment.

use api_client::{apis::account_admin_api::post_update_custom_email, models::UpdateCustomEmail};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_update_custom_email(&no_token_api, UpdateCustomEmail::default()).await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = post_update_custom_email(&account.api(), UpdateCustomEmail::default()).await;
    assert_failure(result)
}

/// Updating a non existing custom email fails.
#[server_test]
async fn updating_non_existing_email_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;

    let result = post_update_custom_email(&admin.api(), UpdateCustomEmail::default()).await;
    assert_failure(result)?;

    Ok(())
}
