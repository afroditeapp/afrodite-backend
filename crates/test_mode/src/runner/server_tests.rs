//! Runner for tests in `server_tests` module

use std::sync::Arc;

use config::{
    args::{QaTestConfig, TestMode},
    Config,
};
use tokio::{select, signal};
use tracing::{error, info};

use crate::{
    client::ApiClient,
    server::{AdditionalSettings, ServerManager},
    TestContext, TestFunction, TestResult,
};

pub mod assert;
pub mod context;

pub struct QaTestRunner {
    config: Arc<Config>,
    test_config: Arc<TestMode>,
    qa_config: QaTestConfig,
}

impl QaTestRunner {
    pub fn new(config: Arc<Config>, test_config: Arc<TestMode>, qa_config: QaTestConfig) -> Self {
        Self {
            config,
            test_config,
            qa_config,
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

        let mut test_context = TestContext::new(self.config.clone(), self.test_config.clone());

        let test_functions: Vec<&'static TestFunction> = match get_test_functions(&self.qa_config) {
            Ok(test_functions) => test_functions,
            Err(()) => return,
        };

        let mut failed = false;
        let mut passed_number = 0;
        let start_time = std::time::Instant::now();

        println!("Running tests...");

        let mut current_test = String::new();
        for test_function in test_functions.iter() {
            current_test = test_function.name();
            print!("test {} ... ", &current_test);
            let manager = ServerManager::new(
                &self.config,
                self.test_config.clone(),
                Some(AdditionalSettings {
                    log_to_memory: true,
                }),
            )
            .await;

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

            test_context.close_websocket_connections().await;
            manager.close().await;

            if failed {
                break;
            } else {
                passed_number += 1;
            }
        }

        let result = if failed { "FAILED" } else { "ok" };

        println!(
            "\ntest result: {}. {} passed; {} failed; {} tests; finished in {:.2?}",
            result,
            passed_number,
            if failed { 1 } else { 0 },
            test_functions.len(),
            start_time.elapsed()
        );

        if failed {
            println!(
                "\nTo continue from the failed test, run command\nmake test CONTINUE_FROM={}",
                current_test
            );
        }
    }
}

fn get_test_functions(test_config: &QaTestConfig) -> Result<Vec<&'static TestFunction>, ()> {
    let mut test_functions: Vec<&'static TestFunction> =
        inventory::iter::<TestFunction>().collect();
    test_functions.sort_by(|a, b| a.name().cmp(&b.name()));

    if let Some(continue_from) = &test_config.continue_from {
        let matching_tests: Vec<&'static TestFunction> = (&test_functions)
            .into_iter()
            .map(|r| *r)
            .filter(|t| t.name().contains(continue_from))
            .collect();
        if matching_tests.is_empty() {
            println!("No tests found containing string: {}", continue_from);
            Err(())
        } else if matching_tests.len() > 1 {
            println!("Unambiguous test selection is required. Found tests:\n");
            for test in matching_tests {
                println!("{}", test.name());
            }
            Err(())
        } else {
            let matching_test = matching_tests[0];
            let matching_test_index = test_functions
                .iter()
                .position(|t| t.name() == matching_test.name())
                .unwrap();
            test_functions = test_functions.split_off(matching_test_index);
            println!("Continuing from test: {}\n", matching_test.name());
            Ok(test_functions)
        }
    } else {
        Ok(test_functions)
    }
}
