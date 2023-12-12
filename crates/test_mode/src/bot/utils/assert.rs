use std::fmt::Debug;

use error_stack::{Result};

use super::super::super::client::TestError;

pub fn bot_assert_eq<T: Debug + PartialEq>(value: T, expected: T) -> Result<(), TestError> {
    if value == expected {
        Ok(())
    } else {
        Err(
            TestError::AssertError(format!("value: {:?}, expected: {:?}", value, expected))
                .report(),
        )
    }
}
