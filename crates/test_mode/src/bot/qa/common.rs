use super::{
    super::actions::account::{Login, Register},
    SingleTest,
};
use crate::{
    bot::actions::{common::TestWebSocket, BotAction},
    test,
};

pub const COMMON_TESTS: &[SingleTest] = &[test!(
    "WebSocket HTTP connection works",
    [Register, Login, TestWebSocket,]
)];
