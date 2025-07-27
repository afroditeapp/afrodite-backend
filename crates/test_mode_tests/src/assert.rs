use core::fmt;

use test_mode_utils::client::TestError;

use crate::{ServerTestError, TestResult};

/// Assert that value is true
#[track_caller]
pub fn assert(value: bool) -> TestResult {
    if value {
        Ok(())
    } else {
        let error = TestError::AssertError(value.to_string());
        Err(ServerTestError::new(error.report()))
    }
}

/// Assert that value equals expected value
#[track_caller]
pub fn assert_eq<T: PartialEq + fmt::Debug>(expect: T, value: T) -> TestResult {
    if expect == value {
        Ok(())
    } else {
        let error = TestError::AssertError(format!("expected: {expect:?}, actual: {value:?}"));
        Err(ServerTestError::new(error.report()))
    }
}

/// Assert that value is not equal to expected value
#[track_caller]
pub fn assert_ne<T: PartialEq + fmt::Debug>(expect: T, value: T) -> TestResult {
    if expect != value {
        Ok(())
    } else {
        let error = TestError::AssertError(format!("expected: {expect:?}, actual: {value:?}"));
        Err(ServerTestError::new(error.report()))
    }
}

#[track_caller]
pub fn assert_failure<Ok, Err>(result: Result<Ok, Err>) -> TestResult {
    if result.is_err() {
        Ok(())
    } else {
        let error = TestError::AssertError("expected: Err, actual: Ok".to_string());
        Err(ServerTestError::new(error.report()))
    }
}
