use api_client::{apis::profile_api::get_location, models::Location};
use test_mode_bot::actions::profile::UpdateLocation;
use test_mode_tests::prelude::*;

const LOCATION_LAT_LON_10: Location = Location {
    latitude: 10.0,
    longitude: 10.0,
};

#[server_test]
async fn location_updates_correctly(mut context: TestContext) -> TestResult {
    let mut account = context.new_account_in_initial_setup_state().await?;
    assert_ne(
        LOCATION_LAT_LON_10,
        get_location(account.account_api()).await?,
    )?;
    account.run(UpdateLocation(LOCATION_LAT_LON_10)).await?;
    assert_eq(
        LOCATION_LAT_LON_10,
        get_location(account.account_api()).await?,
    )
}
