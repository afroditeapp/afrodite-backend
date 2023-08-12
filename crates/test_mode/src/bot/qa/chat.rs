use super::{
    super::actions::account::{Login, Register},
    SingleTest,
};
use crate::{bot::actions::BotAction, test};

pub const CHAT_TESTS: &[SingleTest] = &[test!("TODO", [Register, Login,])];
