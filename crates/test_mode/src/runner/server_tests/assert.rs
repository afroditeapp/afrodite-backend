use core::fmt;

use crate::{client::TestError, TestResult};

/// Assert that value is true
pub fn assert(value: bool) -> TestResult {
    if value {
        Ok(())
    } else {
        let error = TestError::AssertError(format!("{}", value));
        Err(error.report().into())
    }
}

/// Assert that value equals expected value
pub fn assert_eq<T: PartialEq + fmt::Debug>(expect: T, value: T) -> TestResult {
    if expect == value {
        Ok(())
    } else {
        let error = TestError::AssertError(format!("expected: {:?}, actual: {:?}", expect, value));
        Err(error.report().into())
    }
}

pub fn assert_failure<Ok, Err>(result: Result<Ok, Err>) -> TestResult {
    if result.is_err() {
        Ok(())
    } else {
        let error = TestError::AssertError(format!("expected: Err, actual: Ok"));
        Err(error.report().into())
    }
}
