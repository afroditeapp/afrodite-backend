use api_client::apis::account_api::{get_account_state, post_complete_setup};
use test_mode_macro::server_test;

use crate::{
    TestContext, TestResult, action_array,
    bot::actions::{
        account::{AccountState, SetAccountSetup},
        media::{SendImageToSlot, SetContent},
    },
    runner::server_tests::assert::{assert_eq, assert_failure},
};

#[server_test]
async fn account_state_is_initial_setup_after_login(mut context: TestContext) -> TestResult {
    let account = context.new_account_in_initial_setup_state().await?;
    let state = get_account_state(account.account_api()).await?;
    assert_eq(AccountState::InitialSetup, state.into())
}

#[server_test]
async fn complete_setup_fails_if_no_setup_info_is_set(mut context: TestContext) -> TestResult {
    let mut account = context.new_account_in_initial_setup_state().await?;
    account
        .run_actions(action_array![
            SendImageToSlot::slot(0),
            SendImageToSlot::slot(1),
            SetContent {
                security_content_slot_i: Some(0),
                content_0_slot_i: Some(1),
            },
        ])
        .await?;

    assert_failure(post_complete_setup(account.account_api()).await)?;
    assert_eq(
        AccountState::InitialSetup,
        get_account_state(account.account_api()).await?.into(),
    )
}

#[server_test]
async fn initial_setup_successful(mut context: TestContext) -> TestResult {
    let mut account = context.new_account_in_initial_setup_state().await?;
    account
        .run_actions(action_array![
            SetAccountSetup::new(),
            SendImageToSlot::slot(0),
            SendImageToSlot::slot(1),
            SetContent {
                security_content_slot_i: Some(0),
                content_0_slot_i: Some(1),
            },
        ])
        .await?;

    post_complete_setup(account.account_api()).await?;
    assert_eq(
        AccountState::Normal,
        get_account_state(account.account_api()).await?.into(),
    )
}
