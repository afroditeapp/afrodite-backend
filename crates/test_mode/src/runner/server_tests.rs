

//! Runner for tests in `server_tests` module

use std::{path::PathBuf, process::exit, sync::Arc, time::Duration};

use api_client::{apis::configuration::Configuration, manual_additions};
use config::{args::TestMode, Config};
use tokio::{
    io::AsyncWriteExt, select, signal, sync::{mpsc, watch}, time::sleep
};
use tracing::{error, info};
use utils::api;

use crate::{runner::utils::wait_that_servers_start, server::AdditionalSettings, state::StateData, TestFunction};
use crate::{bot::BotManager, client::ApiClient, server::ServerManager, state::BotPersistentState};

pub struct QaTestRunner {
    config: Arc<Config>,
    test_config: Arc<TestMode>,
}

impl QaTestRunner {
    pub fn new(config: Arc<Config>, test_config: Arc<TestMode>) -> Self {
        Self {
            config: config,
            test_config: test_config,
        }
    }

    pub async fn run(self) {
        info!("Testing mode - QA test runner");

        select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(()) => (),
                    Err(e) => {
                        error!("Failed to listen CTRL+C. Error: {}", e);
                    }
                }
            }
            _ = self.run_tests() => (),
        }
    }

    async fn run_tests(&self) {
        let api_urls = Arc::new(self.test_config.server.api_urls.clone());
        let api_client = ApiClient::new(self.test_config.server.api_urls.clone());
        api_client.print_to_log();

        print!("Running tests...\n");
        let test_functions = inventory::iter::<TestFunction>();
        for test_function in test_functions {
            print!("  {}...", test_function.name);

            let manager = ServerManager::new(
                &self.config,
                self.test_config.clone(),
                Some(AdditionalSettings { log_to_memory: true })
            ).await;

            wait_that_servers_start(api_client.clone()).await;

            (test_function.function)();
            println!("OK\n");

            manager.close().await;
        }
    }
}
