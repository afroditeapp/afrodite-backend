use api_client::models::{Location, Profile, ProfileVersion};

use super::SingleTest;
use crate::{
    bot::{actions::{
        account::SetProfileVisibility,
        profile::{GetProfileList, ResetProfileIterator, UpdateLocation, GetProfile, ProfileText, ChangeProfileText},
        AssertEqualsFn, BotAction, ModifyTaskState, RunActions, SleepUntil, TO_NORMAL_STATE, AssertEqualsTestFn,
    }, utils::name::NameProvider},
    test,
};

const LOCATION_LAT_LON_10: Location = Location {
    latitude: 10.0,
    longitude: 10.0,
};

pub const PROFILE_TESTS: &[SingleTest] = &[
    // The next two tests are linked to together.
    // The account in the first test is used in the second test also.
    test!(
        "Update and get location works",
        [
            RunActions(TO_NORMAL_STATE),
            UpdateLocation(LOCATION_LAT_LON_10),
            SetProfileVisibility(true),
            ModifyTaskState(|s| s.bot_count_update_location_to_lat_lon_10 += 1),
            // TODO: Add AssertEqualsFn(
            //    |v, _| v.location(),
            //    LOCATION_LAT_LON_10
            //    &GetLocation),
        ]
    ),
    test!(
        "Get profile changes when visiblity changes",
        [
            RunActions(TO_NORMAL_STATE),
            UpdateLocation(LOCATION_LAT_LON_10),
            SetProfileVisibility(true),
            ModifyTaskState(|s| s.bot_count_update_location_to_lat_lon_10 += 1),
            SleepUntil(|s| s.bot_count_update_location_to_lat_lon_10 >= 2),
            AssertEqualsFn(|v, _| v.profile_count(), 2, &GetProfileList),
            SetProfileVisibility(false),
            ResetProfileIterator,
            AssertEqualsFn(|v, _| v.profile_count(), 1, &GetProfileList),
        ]
    ),
    test!(
        "Get profile",
        [
            RunActions(TO_NORMAL_STATE),
            ChangeProfileText { mode: ProfileText::Static("profile123")},
            AssertEqualsTestFn(
                |v, _| {
                    let mut profile = v.profile().clone();
                    profile.version = ProfileVersion::default().into();
                    profile
                },
                || Profile::new(
                    NameProvider::men_first_name().to_string(),
                    "profile123".to_string(),
                    ProfileVersion::default(),
                ),
                &GetProfile
            ),
        ]
    ),
];

// TODO: Add test for get location route
