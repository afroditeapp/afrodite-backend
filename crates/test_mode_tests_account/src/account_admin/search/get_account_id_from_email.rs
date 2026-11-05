use api_client::apis::account_admin_api::get_account_id_from_email;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = get_account_id_from_email(&no_token_api, "user@example.com").await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = get_account_id_from_email(&account.api(), "user@example.com").await;
    assert_failure(result)
}

/// Account id can be found by email address. Test framework accounts have
/// email addresses starting with "bot".
#[server_test]
async fn account_id_can_be_found_by_email(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let admin = context.new_admin().await?;

    let email = api_client::apis::account_api::get_email_address_state(&account.api())
        .await?
        .email
        .expect("Account created by the test framework has an email");

    let found = get_account_id_from_email(&admin.api(), &email).await?;
    assert_eq(account.account_id(), found.aid.unwrap())?;

    Ok(())
}

/// Non existing email address returns no account id.
#[server_test]
async fn non_existing_email_returns_no_account_id(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;

    let result = get_account_id_from_email(&admin.api(), "nonexisting@example.com").await?;
    assert(result.aid.is_none())?;

    Ok(())
}
