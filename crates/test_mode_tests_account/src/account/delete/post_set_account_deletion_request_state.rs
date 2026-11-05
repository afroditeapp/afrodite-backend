use api_client::apis::account_api::post_set_account_deletion_request_state;
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_set_account_deletion_request_state(
        &no_token_api,
        &account.account_id_string(),
        api_client::models::BooleanSetting::new(true),
    )
    .await;
    assert_failure(result)
}

/// Deletion request of own account can be set and cancelled.
#[server_test]
async fn deletion_request_can_be_set_and_cancelled(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    post_set_account_deletion_request_state(
        &account.api(),
        &account.account_id_string(),
        api_client::models::BooleanSetting::new(true),
    )
    .await?;

    let state = api_client::apis::account_api::get_account_deletion_request_state(
        &account.api(),
        &account.account_id_string(),
    )
    .await?;
    assert(state.automatic_deletion_allowed.is_some())?;

    post_set_account_deletion_request_state(
        &account.api(),
        &account.account_id_string(),
        api_client::models::BooleanSetting::new(false),
    )
    .await?;

    let state = api_client::apis::account_api::get_account_deletion_request_state(
        &account.api(),
        &account.account_id_string(),
    )
    .await?;
    assert(state.automatic_deletion_allowed.is_none())?;

    Ok(())
}

/// Deletion request state of another account cannot be changed without
/// admin permission.
#[server_test]
async fn other_account_deletion_request_fails(mut context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    let account2 = context.new_account().await?;

    let result = post_set_account_deletion_request_state(
        &account1.api(),
        &account2.account_id_string(),
        api_client::models::BooleanSetting::new(true),
    )
    .await;
    assert_failure(result)?;

    let result = api_client::apis::account_api::get_account_deletion_request_state(
        &account1.api(),
        &account2.account_id_string(),
    )
    .await;
    assert_failure(result)?;

    Ok(())
}
