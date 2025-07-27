use std::fmt::Debug;

use error_stack::Result;
use test_mode_utils::client::TestError;

pub fn bot_assert_eq<T: Debug + PartialEq>(value: T, expected: T) -> Result<(), TestError> {
    if value == expected {
        Ok(())
    } else {
        Err(TestError::AssertError(format!("value: {value:?}, expected: {expected:?}")).report())
    }
}
