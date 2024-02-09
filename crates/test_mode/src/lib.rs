#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! Run test suite and benchmarks

pub mod bot;
pub mod client;
mod runner;
mod server;
mod server_tests;
mod state;

use std::{future::Future, sync::Arc};

use client::TestError;
use config::{
    args::{TestMode, TestModeSubMode},
    Config,
};
use error_stack::ResultExt;
use runner::{bot::BotTestRunner, server_tests::QaTestRunner};

pub struct TestRunner {
    config: Arc<Config>,
    test_config: Arc<TestMode>,
}

impl TestRunner {
    pub fn new(config: Config, test_config: TestMode) -> Self {
        Self {
            config: config.into(),
            test_config: test_config.into(),
        }
    }

    pub async fn run(self) {
        tracing_subscriber::fmt::init();

        if let TestModeSubMode::Qa(_) = self.test_config.mode {
            QaTestRunner::new(self.config, self.test_config).run().await;
        } else {
            BotTestRunner::new(self.config, self.test_config)
                .run()
                .await;
        }
    }
}

pub struct TestFunction {
    pub name: &'static str,
    pub module_path: &'static str,
    pub function: fn(TestContext) -> Box<dyn Future<Output = TestResult>>,
}

impl TestFunction {
    pub fn name(&self) -> String {
        let start = self
            .module_path
            .trim_start_matches("test_mode::server_tests::");
        format!("{}::{}", start, self.name)
    }
}

inventory::collect!(TestFunction);

pub use crate::runner::server_tests::context::TestContext;

pub type TestResult = Result<(), ServerTestError>;

/// Workaround for api_client error type conversion to
/// avoid change_context calls.
pub struct ServerTestError {
    pub error: error_stack::Report<TestError>,
}

impl ServerTestError {
    pub fn new(error: error_stack::Report<crate::client::TestError>) -> Self {
        Self { error }
    }
}

impl From<error_stack::Report<crate::client::TestError>> for ServerTestError {
    #[track_caller]
    fn from(error: error_stack::Report<crate::client::TestError>) -> Self {
        Self {
            error: error.change_context(TestError::ServerTestFailed),
        }
    }
}

impl<T> From<api_client::apis::Error<T>> for ServerTestError
where
    api_client::apis::Error<T>: error_stack::Context,
{
    #[track_caller]
    fn from(error: api_client::apis::Error<T>) -> Self {
        Self {
            error: error_stack::Report::from(error).change_context(TestError::ServerTestFailed),
        }
    }
}
