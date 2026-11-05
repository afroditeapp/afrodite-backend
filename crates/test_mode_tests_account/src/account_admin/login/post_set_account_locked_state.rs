use api_client::{
    apis::account_admin_api::post_set_account_locked_state, models::AccountLockedState,
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_set_account_locked_state(
        &no_token_api,
        &admin.account_id_string(),
        AccountLockedState::new(true),
    )
    .await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account1 = context.new_account().await?;
    let account2 = context.new_account().await?;

    let result = post_set_account_locked_state(
        &account1.api(),
        &account2.account_id_string(),
        AccountLockedState::new(true),
    )
    .await;
    assert_failure(result)?;
    let result = post_set_account_locked_state(
        &account1.api(),
        &account1.account_id_string(),
        AccountLockedState::new(true),
    )
    .await;
    assert_failure(result)?;

    Ok(())
}

/// Locked state can be set and reset by admin.
#[server_test]
async fn locked_state_can_be_set_and_reset(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let admin = context.new_admin().await?;

    post_set_account_locked_state(
        &admin.api(),
        &account.account_id_string(),
        AccountLockedState::new(true),
    )
    .await?;

    let state = api_client::apis::account_admin_api::get_account_locked_state(
        &admin.api(),
        &account.account_id_string(),
    )
    .await?;
    assert(state.locked)?;

    post_set_account_locked_state(
        &admin.api(),
        &account.account_id_string(),
        AccountLockedState::new(false),
    )
    .await?;

    let state = api_client::apis::account_admin_api::get_account_locked_state(
        &admin.api(),
        &account.account_id_string(),
    )
    .await?;
    assert(!state.locked)?;

    Ok(())
}
