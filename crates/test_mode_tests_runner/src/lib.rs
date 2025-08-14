#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! Runner for tests marked with [test_mode_tests::server_test] macro.

use std::sync::Arc;

use config::{
    Config,
    args::{QaTestConfig, TestMode},
};
use manager::{ManagerEvent, ManagerEventReceiver, TestManager};
use test_mode_tests::TestFunction;
use test_mode_utils::ServerTestError;
use tokio::{select, signal};
use tracing::{error, info};

mod manager;

pub struct QaTestRunner {
    config: Arc<Config>,
    test_config: Arc<TestMode>,
    qa_config: QaTestConfig,
    reqwest_client: reqwest::Client,
}

impl QaTestRunner {
    pub fn new(
        config: Arc<Config>,
        test_config: Arc<TestMode>,
        qa_config: QaTestConfig,
        reqwest_client: reqwest::Client,
    ) -> Self {
        Self {
            config,
            test_config,
            qa_config,
            reqwest_client,
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
        let test_functions: Vec<&'static TestFunction> = match get_test_functions(&self.qa_config) {
            Ok(test_functions) => test_functions,
            Err(()) => return,
        };

        let (manager_event_receiver, manager_quit_handle) = TestManager::new_manager(
            self.config.clone(),
            self.test_config.clone(),
            test_functions.clone(),
            self.reqwest_client.clone(),
        );

        let mut manager_event_receiver = RunnerUi::new(test_functions, manager_event_receiver)
            .run()
            .await;

        // Wait that all test tasks complete
        let _ = manager_event_receiver.receiver.recv().await;

        manager_quit_handle.wait_quit().await;
    }
}

fn get_test_functions(test_config: &QaTestConfig) -> Result<Vec<&'static TestFunction>, ()> {
    let mut test_functions: Vec<&'static TestFunction> =
        inventory::iter::<TestFunction>().collect();
    test_functions.sort_by_key(|a| a.name());

    if let Some(continue_from) = &test_config.continue_from {
        let matching_tests: Vec<&'static TestFunction> = test_functions
            .iter()
            .copied()
            .filter(|t| t.name().contains(continue_from))
            .collect();
        if matching_tests.is_empty() {
            println!("No tests found containing string: {continue_from}");
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

pub struct RunnerUi {
    test_functions: Vec<&'static TestFunction>,
    manager_event_receiver: ManagerEventReceiver,
}

impl RunnerUi {
    pub fn new(
        test_functions: Vec<&'static TestFunction>,
        manager_event_receiver: ManagerEventReceiver,
    ) -> Self {
        Self {
            test_functions,
            manager_event_receiver,
        }
    }

    pub async fn run(mut self) -> ManagerEventReceiver {
        let mut failed = false;
        let mut passed_number = 0;
        let start_time = std::time::Instant::now();

        println!("Running tests...");

        let mut pending_events = vec![];
        let mut current_test = String::new();
        for test_function in self.test_functions.iter() {
            current_test = test_function.name();
            print!("test {} ... ", &current_test);

            let result = wait_that_correct_test_event_is_received(
                &mut self.manager_event_receiver,
                &current_test,
                &mut pending_events,
            )
            .await;

            match result {
                Ok(()) => println!("ok"),
                Err(e) => {
                    failed = true;
                    println!("FAILED\n");
                    println!("Test failed: {:?}\n", e.error.error);
                    println!("{}", e.logs);
                }
            }

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
            self.test_functions.len(),
            start_time.elapsed()
        );

        if failed {
            println!(
                "\nTo continue from the failed test, run command\nmake test CONTINUE_FROM={current_test}"
            );
        }

        self.manager_event_receiver
    }
}

async fn wait_that_correct_test_event_is_received(
    receiver: &mut ManagerEventReceiver,
    test_name: &str,
    pending: &mut Vec<ManagerEvent>,
) -> Result<(), ErrorInfo> {
    for (i, pending_event) in pending.iter().enumerate() {
        if pending_event.test().name() == test_name {
            let event = pending.remove(i);
            return handle_event(event);
        }
    }

    loop {
        match receiver.receiver.recv().await {
            Some(e) => {
                if e.test().name() == test_name {
                    return handle_event(e);
                } else {
                    pending.push(e);
                }
            }
            None => panic!("ManagerEventReceiver closed"),
        }
    }
}

fn handle_event(e: ManagerEvent) -> Result<(), ErrorInfo> {
    match e {
        ManagerEvent::Success { .. } => Ok(()),
        ManagerEvent::Fail { error, logs, .. } => Err(ErrorInfo { error, logs }),
    }
}

struct ErrorInfo {
    error: ServerTestError,
    logs: String,
}
