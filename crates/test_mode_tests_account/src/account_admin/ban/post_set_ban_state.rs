use api_client::{apis::account_admin_api::post_set_ban_state, models::SetAccountBanState};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let admin = context.new_admin().await?;
    let mut no_token_api = (*admin.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = post_set_ban_state(
        &no_token_api,
        SetAccountBanState::new(api_client::models::AccountId::default()),
    )
    .await;
    assert_failure(result)
}

/// Request without admin permission fails.
#[server_test]
async fn request_without_admin_permission_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    let result = post_set_ban_state(
        &account.api(),
        SetAccountBanState::new(account.account_id()),
    )
    .await;
    assert_failure(result)
}

/// Account can be banned by admin and the ban is visible to the banned
/// account.
#[server_test]
async fn account_can_be_banned(context: TestContext) -> TestResult {
    let mut context = context;
    let account = context.new_account().await?;
    let admin = context.new_admin().await?;

    let mut ban_state = SetAccountBanState::new(account.account_id());
    ban_state.ban_until = Some(api_client::models::UnixTime::new(4_102_444_800_000));
    post_set_ban_state(&admin.api(), ban_state).await?;

    let ban_time = api_client::apis::account_api::get_account_ban_time(
        &account.api(),
        &account.account_id_string(),
    )
    .await?;
    assert(ban_time.banned_until.is_some())?;

    Ok(())
}

/// Account can be unbanned by admin.
#[server_test]
async fn account_can_be_unbanned(context: TestContext) -> TestResult {
    let mut context = context;
    let account = context.new_account().await?;
    let admin = context.new_admin().await?;

    let mut ban_state = SetAccountBanState::new(account.account_id());
    ban_state.ban_until = Some(api_client::models::UnixTime::new(4_102_444_800_000));
    post_set_ban_state(&admin.api(), ban_state).await?;

    post_set_ban_state(&admin.api(), SetAccountBanState::new(account.account_id())).await?;

    let ban_time = api_client::apis::account_api::get_account_ban_time(
        &account.api(),
        &account.account_id_string(),
    )
    .await?;
    assert(ban_time.banned_until.is_none())?;

    Ok(())
}
