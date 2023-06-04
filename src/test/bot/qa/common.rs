use api_client::models::Location;

use crate::test::bot::actions::{
    account::SetProfileVisibility,
    common::TestWebSocket,
    profile::{GetProfileList, ResetProfileIterator, UpdateLocation},
    ActionArray, AssertEquals, AssertEqualsFn, BotAction, ModifyTaskState, RunActions, SleepUntil,
    TO_NORMAL_STATE,
};

use super::{
    super::actions::{
        account::{Login, Register},
        media::SendImageToSlot,
        AssertFailure,
    },
    SingleTest,
};

use crate::test;

pub const COMMON_TESTS: &[SingleTest] = &[test!(
    "WebSocket HTTP connection works",
    [Register, Login, TestWebSocket,]
)];
