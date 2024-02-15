


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
    runner::server_tests::assert::{assert_eq, assert_failure, assert_ne},
    TestContext, TestResult,
};

const LOCATION_LAT_LON_10: Location = Location {
    latitude: 10.0,
    longitude: 10.0,
};

#[server_test]
async fn location_updates_correctly(context: TestContext) -> TestResult {
    let mut account = context.new_account_in_initial_setup_state().await?;
    assert_ne(
        LOCATION_LAT_LON_10,
        get_location(account.account_api()).await?
    )?;
    account.run(UpdateLocation(LOCATION_LAT_LON_10)).await?;
    assert_eq(
        LOCATION_LAT_LON_10,
        get_location(account.account_api()).await?
    )
}
