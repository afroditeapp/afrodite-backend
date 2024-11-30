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
    ContentSlot,
);

disable_logging!(
    // Account
    GoogleAccountId,
    // Chat
    // MessageNumber,
    // // General
    // i64,
    // (),
);
