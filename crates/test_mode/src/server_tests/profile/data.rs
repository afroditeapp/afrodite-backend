


use api_client::{
    apis::{account_api::{get_account_state, post_complete_setup}, profile_api::{get_location, get_profile, post_get_next_profile_page, post_profile}},
    models::{account, AccountState, Location, ProfileUpdate, ProfileAttributeValueUpdate, ProfileAge},
};
use test_mode_macro::server_test;

use crate::{
    action_array,
    bot::actions::{
        account::{SetAccountSetup, SetProfileVisibility}, media::{MakeModerationRequest, SendImageToSlot, SetPendingContent}, profile::UpdateLocation, AssertFailure
    },
    runner::server_tests::assert::{assert_eq, assert_failure, assert_ne},
    TestContext, TestResult,
};

#[server_test]
async fn updating_profile_works(context: TestContext) -> TestResult {
    let account = context.new_account_in_initial_setup_state().await?;
    let profile = ProfileUpdate {
        attributes: vec![],
        age: 18,
        name: format!(""),
        profile_text: format!("test"),
    };
    post_profile(account.account_api(), profile)
        .await?;
    assert_eq(
        "test",
        &get_profile(account.account_api(), &account.account_id_string()).await?.profile_text
    )
}
