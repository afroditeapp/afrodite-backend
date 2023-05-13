use crate::test::bot::actions::BotAction;

use super::{
    super::actions::{
        account::{Login, Register},
        media::SendImageToSlot,
        AssertFailure,
    },
    SingleTest,
};

use crate::test;

pub const MEDIA_TESTS: &[SingleTest] = &[
    test!(
        "Save image to slot: max 3 slots",
        [
            Register,
            Login,
            AssertFailure(SendImageToSlot(3)),
        ]
    )
];
