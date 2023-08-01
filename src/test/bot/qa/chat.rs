use crate::test::bot::actions::BotAction;

use super::{
    super::actions::{
        account::{Login, Register},
    },
    SingleTest,
};

use crate::test;

pub const CHAT_TESTS: &[SingleTest] = &[test!(
    "TODO",
    [Register, Login,]
)];
