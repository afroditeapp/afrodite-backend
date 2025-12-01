use model::{disable_logging, enable_logging};

use super::*;

enable_logging!(
    // Account
);

disable_logging!(
    // Account
    AppleAccountId,
    GoogleAccountId,
);
