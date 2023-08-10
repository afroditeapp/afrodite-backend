use crate::bot::actions::{common::TestWebSocket, BotAction};

use super::{
    super::actions::account::{Login, Register},
    SingleTest,
};

use crate::test;

pub const COMMON_TESTS: &[SingleTest] = &[test!(
    "WebSocket HTTP connection works",
    [Register, Login, TestWebSocket,]
)];
