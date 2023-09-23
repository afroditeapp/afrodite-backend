#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! Run test suite and benchmarks

mod bot;
pub mod client;
mod server;
mod state;

use std::{path::PathBuf, sync::Arc, time::Duration};

use api_client::{apis::configuration::Configuration, manual_additions};
use config::{args::TestMode, Config};
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

        info!("Testing mode");

        let old_state = if self.test_config.save_state() {
            self.load_state_data().await.map(|d| Arc::new(d))
        } else {
            None
        };

        ApiClient::new(self.test_config.server.api_urls.clone()).print_to_log();

        let server = if !self.test_config.no_servers {
            Some(ServerManager::new(&self.config, self.test_config.clone()).await)
        } else {
            None
        };

        let (bot_running_handle, mut wait_all_bots) = mpsc::channel::<Vec<BotPersistentState>>(1);
        let (quit_handle, bot_quit_receiver) = watch::channel(());

        let mut task_number = 0;
        let api_urls = Arc::new(self.test_config.server.api_urls.clone());

        info!("Waiting API availability...");

        let quit_now = select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(()) => true,
                    Err(e) => {
                        error!("Failed to listen CTRL+C. Error: {}", e);
                        true
                    }
                }
            }
            _ = wait_that_servers_start(ApiClient::new(api_urls.as_ref().clone())) => {
                false
            },
        };

        if !quit_now {
            info!("...API ready");

            info!(
                "Task count: {}, Bot count per task: {}",
                self.test_config.tasks(), self.test_config.bots(),
            );

            while task_number < self.test_config.tasks() {
                BotManager::spawn(
                    task_number,
                    self.config.clone(),
                    self.test_config.clone(),
                    old_state.clone(),
                    bot_quit_receiver.clone(),
                    bot_running_handle.clone(),
                );
                task_number += 1;
            }

            info!("Bot tasks are now created",);
        }

        drop(bot_running_handle);
        drop(bot_quit_receiver);

        select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(()) => (),
                    Err(e) => error!("Failed to listen CTRL+C. Error: {}", e),
                }
            }
            _ = wait_all_bots.recv() => ()
        }

        drop(quit_handle); // Singnal quit to bots.

        // Wait that all bot_running_handles are dropped.
        let mut bot_states = vec![];
        loop {
            match wait_all_bots.recv().await {
                None => break,
                Some(data) => bot_states.extend(data),
            }
        }

        let new_state = StateData {
            test_name: self.test_config.test_name(),
            bot_states,
        };

        if self.test_config.save_state() {
            self.save_state_data(&new_state).await;
        }

        // Quit
        if let Some(server) = server {
            server.close().await;
        }
    }

    async fn load_state_data(&self) -> Option<StateData> {
        match tokio::fs::read_to_string(self.state_data_file()).await {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(data) => Some(data),
                Err(e) => {
                    error!("state data loading error: {:?}", e);
                    None
                }
            },
            Err(e) => {
                error!("state data loading error: {:?}", e);
                None
            }
        }
    }

    async fn save_state_data(&self, data: &StateData) {
        let data = match serde_json::to_string_pretty(data) {
            Ok(d) => d,
            Err(e) => {
                error!("state saving error: {:?}", e);
                return;
            }
        };

        let file_handle = tokio::fs::File::create(self.state_data_file()).await;

        match file_handle {
            Ok(mut handle) => match handle.write_all(data.as_bytes()).await {
                Ok(()) => (),
                Err(e) => {
                    error!("state data saving error: {:?}", e);
                }
            },
            Err(e) => {
                error!("state data saving error: {:?}", e);
            }
        }
    }

    fn state_data_file(&self) -> PathBuf {
        let data_file =
            format!("test_{}_state_data.json", self.test_config.test_name());
        if !self.test_config.server.test_database.exists() {
            std::fs::create_dir_all(&self.test_config.server.test_database)
                .unwrap();
        }
        self.test_config.server.test_database.join(data_file)
    }
}

async fn wait_that_servers_start(api: ApiClient) {
    check_api(api.account()).await;
    check_api(api.profile()).await;
    check_api(api.media()).await;
    check_api(api.chat()).await;
}

async fn check_api(config: &Configuration) {
    loop {
        match manual_additions::api_available(config).await {
            Ok(()) => break,
            Err(()) => (),
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
