use std::sync::{Arc, atomic::AtomicBool};

use config::{Config, args::TestMode};
use test_mode_utils::server::{AdditionalSettings, ServerManager};
use tokio::{
    sync::{self, OwnedSemaphorePermit, mpsc},
    task::JoinHandle,
};

use crate::{ServerTestError, TestContext, TestFunction, TestResult};

static TEST_FAILURE: AtomicBool = AtomicBool::new(false);

pub enum ManagerEvent {
    Success {
        test: &'static TestFunction,
    },
    Fail {
        test: &'static TestFunction,
        error: ServerTestError,
        logs: String,
    },
}

impl ManagerEvent {
    pub fn test(&self) -> &'static TestFunction {
        match self {
            Self::Success { test } | Self::Fail { test, .. } => test,
        }
    }
}

pub struct ManagerQuitHandle {
    task: JoinHandle<()>,
}

impl ManagerQuitHandle {
    pub async fn wait_quit(self) {
        self.task.await.unwrap();
    }
}
pub struct ManagerEventReceiver {
    pub receiver: mpsc::Receiver<ManagerEvent>,
}

pub struct TestManager {
    config: Arc<Config>,
    test_config: Arc<TestMode>,
    test_functions: Vec<&'static TestFunction>,
    reqwest_client: reqwest::Client,
}

impl TestManager {
    pub fn new_manager(
        config: Arc<Config>,
        test_config: Arc<TestMode>,
        test_functions: Vec<&'static TestFunction>,
        reqwest_client: reqwest::Client,
    ) -> (ManagerEventReceiver, ManagerQuitHandle) {
        let manager = Self {
            config,
            test_config,
            test_functions,
            reqwest_client,
        };

        let (sender, receiver) = mpsc::channel(10);
        let receiver = ManagerEventReceiver { receiver };
        let task = tokio::spawn(manager.run_tests(sender));
        let quit_handle = ManagerQuitHandle { task };

        (receiver, quit_handle)
    }

    async fn run_tests(self, sender: mpsc::Sender<ManagerEvent>) {
        let task_count = self
            .test_config
            .qa_mode()
            .as_ref()
            .unwrap()
            .tasks
            .unwrap_or(num_cpus::get());
        let semaphore = Arc::new(sync::Semaphore::new(task_count));

        let max_port_number_count = task_count * 2;
        let (port_number_sender, mut port_number_receiver) =
            port_number_channel(max_port_number_count);
        let initial_port_number = 3100;
        for port_number in initial_port_number..initial_port_number + max_port_number_count as u16 {
            port_number_sender.send(port_number).await;
        }

        for test in self.test_functions.into_iter() {
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            if TEST_FAILURE.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            let api_port = port_number_receiver.receive().await;

            let test_task = TestTask {
                config: self.config.clone(),
                test_config: self.test_config.clone(),
                permit,
                sender: sender.clone(),
                port_sender: port_number_sender.clone(),
                test,
                api_port,
                reqwest_client: self.reqwest_client.clone(),
            };
            tokio::spawn(test_task.run());
        }
    }
}

struct TestTask {
    config: Arc<Config>,
    test_config: Arc<TestMode>,
    permit: OwnedSemaphorePermit,
    sender: mpsc::Sender<ManagerEvent>,
    port_sender: ServerPortNumberSender,
    test: &'static TestFunction,
    api_port: u16,
    reqwest_client: reqwest::Client,
}

impl TestTask {
    pub async fn run(self) {
        let mut test_context = TestContext::new(
            self.config.clone(),
            self.test_config.clone(),
            Some(self.api_port),
            self.reqwest_client,
        );

        let server_manager = ServerManager::new(
            &self.config,
            self.test_config.clone(),
            Some(AdditionalSettings {
                log_to_memory: true,
                account_server_api_port: Some(self.api_port),
            }),
        )
        .await;

        let test_future = (self.test.function)(test_context.clone());
        let test_future =
            Box::<dyn futures::Future<Output = TestResult> + Send>::into_pin(test_future);

        match test_future.await {
            Ok(_) => {
                self.sender
                    .send(ManagerEvent::Success { test: self.test })
                    .await
                    .unwrap();
            }
            Err(error) => {
                TEST_FAILURE.store(true, std::sync::atomic::Ordering::Relaxed);
                self.sender
                    .send(ManagerEvent::Fail {
                        test: self.test,
                        error,
                        logs: server_manager.logs_string().await,
                    })
                    .await
                    .unwrap();
            }
        }

        test_context.close_websocket_connections().await;
        server_manager.close().await;

        self.port_sender.send(self.api_port).await;

        drop(self.permit);
    }
}

fn port_number_channel(size: usize) -> (ServerPortNumberSender, ServerPortNumberReceiver) {
    let (sender, receiver) = mpsc::channel(size);
    (
        ServerPortNumberSender { sender },
        ServerPortNumberReceiver { receiver },
    )
}

/// Send free port number back to the manager.
#[derive(Clone)]
struct ServerPortNumberSender {
    sender: mpsc::Sender<u16>,
}

impl ServerPortNumberSender {
    pub async fn send(&self, port_number: u16) {
        let _ = self.sender.send(port_number).await;
    }
}

struct ServerPortNumberReceiver {
    receiver: mpsc::Receiver<u16>,
}

impl ServerPortNumberReceiver {
    pub async fn receive(&mut self) -> u16 {
        self.receiver.recv().await.unwrap()
    }
}
