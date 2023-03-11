//! Run test suite and benchmarks

use std::sync::Arc;

use tracing::info;

use crate::{config::{Config, args::TestMode}, server::database::DatabaseManager};


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

        info!("Testing mode");
    }
}
