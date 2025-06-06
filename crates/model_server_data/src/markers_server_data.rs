use model::{disable_logging, enable_logging};

use super::*;

enable_logging!(
    // Account
    // AccountIdInternal,
    // AccountId,
    // Option<AccountIdDb>,
    // // Media
    // ContentId,
    // Option<ContentId>,
    // ModerationQueueNumber,
    // ModerationQueueType,
    // NextQueueNumberType,
);

disable_logging!(
    // Account
    AppleAccountId,
    GoogleAccountId,
    // Chat
    // MessageNumber,
    // // General
    // i64,
    // (),
);
