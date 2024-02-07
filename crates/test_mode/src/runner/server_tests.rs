

//! Runner for tests in `server_tests` module

use std::{panic::UnwindSafe, path::PathBuf, process::exit, sync::Arc, time::Duration};

use api_client::{apis::configuration::Configuration, manual_additions};
use config::{args::TestMode, Config};
use futures::{FutureExt, TryFutureExt};
use tokio::{
    io::AsyncWriteExt, select, signal, sync::{mpsc, watch}, time::sleep
};
use tracing::{error, info};
use utils::api;

use crate::{runner::utils::wait_that_servers_start, server::AdditionalSettings, state::StateData, TestContext, TestFunction, TestResult};
use crate::{bot::BotManager, client::ApiClient, server::ServerManager, state::BotPersistentState};

pub mod context;

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
        let api_client = ApiClient::new(self.test_config.server.api_urls.clone());
        api_client.print_to_log();

        let mut test_context = TestContext::new(
            self.config.clone(),
            self.test_config.clone()
        );

        let mut failed = false;
        let mut passed_number = 0;
        let start_time = std::time::Instant::now();

        print!("Running tests...\n");
        let mut test_functions: Vec<&'static TestFunction> = inventory::iter::<TestFunction>().collect();
        test_functions.sort_by(|a, b| {
            a.name().cmp(&b.name())
        });

        for test_function in test_functions {
            print!("test {} ... ", test_function.name());

            let manager = ServerManager::new(
                &self.config,
                self.test_config.clone(),
                Some(AdditionalSettings { log_to_memory: true })
            ).await;

            wait_that_servers_start(api_client.clone()).await;

            let test_future = (test_function.function)(test_context.clone());
            let test_future =
                Box::<dyn futures::Future<Output = TestResult>>::into_pin(test_future);

            match test_future.await {
                Ok(()) => println!("ok"),
                Err(e) => {
                    failed = true;
                    println!("FAILED\n");
                    println!("Test failed: {:?}\n", e.error);
                    println!("{}", manager.logs_string().await);
                }
            }

            manager.close().await;
            test_context.clear().await;

            if failed {
                break;
            } else {
                passed_number += 1;
            }
        }

        let result = if failed {
            "FAILED"
        } else {
            "ok"
        };

        println!(
            "\ntest result: {}. {} passed; {} failed; finished in {:.2?}\n\n",
            result,
            passed_number,
            if failed { 1 } else { 0 },
            start_time.elapsed()
        );
    }
}
