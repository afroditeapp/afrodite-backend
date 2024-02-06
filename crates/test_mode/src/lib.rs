#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! Run test suite and benchmarks

mod bot;
pub mod client;
mod server;
mod state;
mod server_tests;
mod runner;

use std::{path::PathBuf, sync::Arc, time::Duration};

use api_client::{apis::configuration::Configuration, manual_additions};
use config::{args::{TestMode, TestModeSubMode}, Config};
use runner::{bot::BotTestRunner, server_tests::QaTestRunner};
use tokio::{
    io::AsyncWriteExt,
    select, signal,
    sync::{mpsc, watch},
};
use tracing::{error, info};

use self::state::StateData;
use crate::{bot::BotManager, client::ApiClient, server::ServerManager, state::BotPersistentState};

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
            BotTestRunner::new(self.config, self.test_config).run().await;
        }
    }
}

pub struct TestFunction {
    pub name: &'static str,
    pub module_path: &'static str,
    pub function: fn(),
}

impl TestFunction {
    pub const fn new(name: &'static str, module_path: &'static str, function: fn()) -> Self {
        Self {
            name,
            module_path,
            function,
        }
    }
}

inventory::collect!(TestFunction);
