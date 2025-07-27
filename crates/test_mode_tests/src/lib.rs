#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use test_mode_utils::ServerTestError;

mod assert;
mod context;
pub mod prelude;

pub use test_mode_macro::server_test;

pub use crate::context::TestContext;

/// [server_test] requires this
pub type TestResult = Result<(), ServerTestError>;

/// [server_test] requires this
pub struct TestFunction {
    pub name: &'static str,
    pub module_path: &'static str,
    pub function: fn(TestContext) -> Box<dyn Future<Output = TestResult> + Send>,
}

/// [server_test] requires this
impl TestFunction {
    pub fn name(&self) -> String {
        let start = self
            .module_path
            .trim_start_matches("test_mode::server_tests::");
        format!("{}::{}", start, self.name)
    }
}

inventory::collect!(TestFunction);
