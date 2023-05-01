


use crate::test::bot::actions::{BotAction, ActionArray, TO_NORMAL_STATE, AssertEquals, profile::{UpdateLocation, GetProfiles, ResetProfileIterator}, RunActions, ModifyTaskState, SleepUntil, AssertEqualsFn, account::SetProfileVisibility};

use super::{
    super::actions::{
        account::{Login, Register},
        media::SendImageToSlot,
        AssertFailure,
    },
    SingleTest,
};

use crate::test;

pub const PROFILE_TESTS: &[SingleTest] = &[
    test!(
        "Update location works",
        [
            RunActions(TO_NORMAL_STATE),
            UpdateLocation { lat: 10.0, lon: 10.0 },
            SetProfileVisibility(true),
            ModifyTaskState(|s| s.bot_count_update_location_to_lat_lon_10 += 1),
        ]
    ),
    test!(
        "Get profile changes when visiblity changes",
        [
            RunActions(TO_NORMAL_STATE),
            UpdateLocation { lat: 10.0, lon: 10.0 },
            SetProfileVisibility(true),
            ModifyTaskState(|s| s.bot_count_update_location_to_lat_lon_10 += 1),
            SleepUntil(|s| s.bot_count_update_location_to_lat_lon_10 >= 2),
            AssertEqualsFn(|v, _| v.profile_count(), 2, &GetProfiles),
            SetProfileVisibility(false),
            ResetProfileIterator,
            AssertEqualsFn(|v, _| v.profile_count(), 1, &GetProfiles),
        ]
    )
];
