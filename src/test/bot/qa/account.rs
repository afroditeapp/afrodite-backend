use api_client::models::AccountState;

use crate::test::bot::actions::BotAction;

use super::{
    super::actions::{
        account::{AssertAccountState, CompleteAccountSetup, Login, Register, SetAccountSetup},
        media::{MakeModerationRequest, SendImageToSlot},
        AssertFailure,
    },
    SingleTest,
};

use crate::test;

pub const ACCOUNT_TESTS: &[SingleTest] = &[
    test!(
        "Initial setup: correct account state after login",
        [
            Register,
            Login,
            AssertAccountState(AccountState::InitialSetup),
        ]
    ),
    test!(
        "Initial setup: complete setup fails if no setup info is set",
        [
            Register,
            Login,
            SendImageToSlot::slot(0),
            SendImageToSlot::slot(1),
            MakeModerationRequest { camera: true },
            AssertFailure(CompleteAccountSetup),
            AssertAccountState(AccountState::InitialSetup),
        ]
    ),
    test!(
        "Initial setup: complete setup fails if no image moderation request",
        [
            Register,
            Login,
            SetAccountSetup::new(),
            AssertFailure(CompleteAccountSetup),
            AssertAccountState(AccountState::InitialSetup),
        ]
    ),
    test!(
        "Initial setup: complete setup fails if image request does not contain camera image",
        [
            Register,
            Login,
            SetAccountSetup::new(),
            SendImageToSlot::slot(0),
            SendImageToSlot::slot(1),
            MakeModerationRequest { camera: false },
            AssertFailure(CompleteAccountSetup),
            AssertAccountState(AccountState::InitialSetup),
        ]
    ),
    test!(
        "Initial setup: successful",
        [
            Register,
            Login,
            SetAccountSetup::new(),
            SendImageToSlot::slot(0),
            SendImageToSlot::slot(1),
            MakeModerationRequest { camera: true },
            CompleteAccountSetup,
            AssertAccountState(AccountState::Normal),
        ]
    ),
];
