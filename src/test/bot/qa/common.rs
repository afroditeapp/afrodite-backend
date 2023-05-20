




use api_client::models::Location;

use crate::test::bot::actions::{BotAction, ActionArray, TO_NORMAL_STATE, AssertEquals, profile::{UpdateLocation, GetProfileList, ResetProfileIterator}, RunActions, ModifyTaskState, SleepUntil, AssertEqualsFn, account::SetProfileVisibility, common::TestWebSocket};

use super::{
    super::actions::{
        account::{Login, Register},
        media::SendImageToSlot,
        AssertFailure,
    },
    SingleTest,
};

use crate::test;

pub const COMMON_TESTS: &[SingleTest] = &[
    test!(
        "WebSocket HTTP connection works",
        [
            Register,
            Login,
            TestWebSocket,
        ]
    ),
];
