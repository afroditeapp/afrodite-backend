
use api_client::{apis::account_api::{get_account_state, post_complete_setup}, models::AccountState};
use test_mode_macro::server_test;

use crate::{action_array, bot::actions::{account::SetAccountSetup, media::{MakeModerationRequest, SendImageToSlot}}, runner::server_tests::assert::{assert_eq, assert_failure}, TestContext, TestResult};


#[server_test]
async fn account_state_is_initial_setup_after_login(context: TestContext) -> TestResult {
    let account = context.new_account().await?;
    let state = get_account_state(account.account_api()).await?;
    assert_eq(AccountState::InitialSetup, state.state)
}

#[server_test]
async fn complete_setup_fails_if_no_setup_info_is_set(context: TestContext) -> TestResult {
    let mut account = context.new_account().await?;
    account.run_actions(action_array![
        SendImageToSlot::slot(0),
        SendImageToSlot::slot(1),
        MakeModerationRequest { camera: true },
    ]).await?;

    assert_failure(post_complete_setup(account.account_api()).await)?;
    assert_eq(
        AccountState::InitialSetup,
        get_account_state(account.account_api()).await?.state,
    )
}

#[server_test]
async fn complete_setup_fails_if_no_image_moderation_request(context: TestContext) -> TestResult {
    let mut account = context.new_account().await?;
    account.run(SetAccountSetup::new()).await?;

    assert_failure(post_complete_setup(account.account_api()).await)?;
    assert_eq(
        AccountState::InitialSetup,
        get_account_state(account.account_api()).await?.state,
    )
}

#[server_test]
async fn complete_setup_fails_if_image_request_does_not_contain_camera_image(context: TestContext) -> TestResult {
    let mut account = context.new_account().await?;
    account.run_actions(action_array![
        SetAccountSetup::new(),
        SendImageToSlot::slot(0),
        SendImageToSlot::slot(1),
        MakeModerationRequest { camera: false },
    ]).await?;

    assert_failure(post_complete_setup(account.account_api()).await)?;
    assert_eq(
        AccountState::InitialSetup,
        get_account_state(account.account_api()).await?.state,
    )
}

#[server_test]
async fn initial_setup_successful(context: TestContext) -> TestResult {
    let mut account = context.new_account().await?;
    account.run_actions(action_array![
        SetAccountSetup::new(),
        SendImageToSlot::slot(0),
        SendImageToSlot::slot(1),
        MakeModerationRequest { camera: true },
    ]).await?;

    assert_failure(post_complete_setup(account.account_api()).await)?;
    assert_eq(
        AccountState::Normal,
        get_account_state(account.account_api()).await?.state,
    )
}
