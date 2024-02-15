


use api_client::{
    apis::{account_api::{get_account_state, post_complete_setup}, profile_api::{get_location, post_get_next_profile_page}},
    models::{account, AccountState, Location},
};
use test_mode_macro::server_test;

use crate::{
    action_array,
    bot::actions::{
        account::{SetAccountSetup, SetProfileVisibility}, media::{MakeModerationRequest, SendImageToSlot, SetPendingContent}, profile::UpdateLocation, AssertFailure
    },
    runner::server_tests::assert::{assert_eq, assert_failure},
    TestContext, TestResult,
};

#[server_test]
async fn visiblity_capability_updates(context: TestContext) -> TestResult {
    let mut account = context.new_account_in_initial_setup_state().await?;
    assert_eq(
        false,
        get_account_state(account.account_api())
            .await?
            .capabilities
            .user_view_public_profiles
            .unwrap_or_default()
    )?;
    account.run(SetProfileVisibility(true)).await?;
    assert_eq(
        true,
        get_account_state(account.account_api())
            .await?
            .capabilities
            .user_view_public_profiles
            .unwrap_or_default()
    )
}

#[server_test]
async fn visiblity_changes_available_profiles(context: TestContext) -> TestResult {
    let mut account1 = context.new_account_in_initial_setup_state().await?;
    let mut account2 = context.new_account_in_initial_setup_state().await?;
    assert_eq(
        0,
        post_get_next_profile_page(account1.profile_api()).await?.profiles.len()
    )?;
    account1.run(SetProfileVisibility(true)).await?;
    assert_eq(
        1,
        post_get_next_profile_page(account1.profile_api()).await?.profiles.len()
    )?;
    account2.run(SetProfileVisibility(true)).await?;
    assert_eq(
        2,
        post_get_next_profile_page(account1.profile_api()).await?.profiles.len()
    )?;
    account1.run(SetProfileVisibility(false)).await?;
    assert_eq(
        1,
        post_get_next_profile_page(account1.profile_api()).await?.profiles.len()
    )?;
    account2.run(SetProfileVisibility(false)).await?;
    assert_eq(
        0,
        post_get_next_profile_page(account1.profile_api()).await?.profiles.len()
    )
}
