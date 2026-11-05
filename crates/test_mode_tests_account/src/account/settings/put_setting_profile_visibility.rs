use api_client::{
    apis::account_api::{get_account_state, put_setting_profile_visibility},
    models::BooleanSetting,
};
use test_mode_test_utils::prelude::*;

/// API requires access token.
#[server_test]
async fn unauthorized_request_fails(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let mut no_token_api = (*account.api()).clone();
    no_token_api.bearer_access_token = None;

    let result = put_setting_profile_visibility(&no_token_api, BooleanSetting::new(false)).await;
    assert_failure(result)
}

/// Profile visibility can be changed and read back from account state.
#[server_test]
async fn profile_visibility_can_be_changed(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;

    put_setting_profile_visibility(&account.api(), BooleanSetting::new(false)).await?;
    assert_eq(
        api_client::models::ProfileVisibility::Private,
        get_account_state(&account.api()).await?.visibility,
    )?;

    put_setting_profile_visibility(&account.api(), BooleanSetting::new(true)).await?;
    assert_eq(
        api_client::models::ProfileVisibility::Public,
        get_account_state(&account.api()).await?.visibility,
    )?;

    Ok(())
}

/// Banned account cannot set visibility to public.
#[server_test]
async fn banned_account_cannot_set_public_visibility(mut context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let admin = context.new_admin().await?;

    let mut ban_state = api_client::models::SetAccountBanState::new(account.account_id());
    ban_state.ban_until = Some(api_client::models::UnixTime::new(4_102_444_800_000));

    api_client::apis::account_admin_api::post_set_ban_state(&admin.api(), ban_state).await?;

    // Banning already sets visibility to private.
    assert_eq(
        api_client::models::ProfileVisibility::Private,
        get_account_state(&account.api()).await?.visibility,
    )?;

    let result = put_setting_profile_visibility(&account.api(), BooleanSetting::new(true)).await;
    assert_failure(result)?;

    // Setting private is still allowed.
    put_setting_profile_visibility(&account.api(), BooleanSetting::new(false)).await?;

    let mut unban_state = api_client::models::SetAccountBanState::new(account.account_id());
    unban_state.ban_until = None;
    api_client::apis::account_admin_api::post_set_ban_state(&admin.api(), unban_state).await?;

    // After unban visibility can be set to public again.
    put_setting_profile_visibility(&account.api(), BooleanSetting::new(true)).await?;
    assert_eq(
        api_client::models::ProfileVisibility::Public,
        get_account_state(&account.api()).await?.visibility,
    )?;

    Ok(())
}
