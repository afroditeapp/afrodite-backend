#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! Admin bot, user bots and benchmarks

mod actions;
mod admin_bot;
mod benchmark_bot;
mod user_bot;
mod utils;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use config::{args::TestMode, bot_config_file::BotConfigFile};
use test_mode_utils::{
    client::ApiClient,
    dir::DataDirUtils,
    server::{ServerInstanceConfig, ServerManager},
    state::{BotPersistentState, StateData},
};
use tokio::{
    io::AsyncWriteExt,
    select, signal,
    sync::{mpsc, watch},
};
use tracing::{error, info};

use crate::{admin_bot::AdminBot, benchmark_bot::BenchmarkBot, user_bot::UserBot};

pub struct BotTestRunner {
    server_instance_config: ServerInstanceConfig,
    bot_config_file: Arc<BotConfigFile>,
    test_config: Arc<TestMode>,
    reqwest_client: reqwest::Client,
}

impl BotTestRunner {
    pub fn new(
        server_instance_config: ServerInstanceConfig,
        bot_config_file: Arc<BotConfigFile>,
        test_config: Arc<TestMode>,
        reqwest_client: reqwest::Client,
    ) -> Self {
        Self {
            server_instance_config,
            bot_config_file,
            test_config,
            reqwest_client,
        }
    }

    pub async fn run(self) {
        info!("Testing mode - bot test runner");
        info!("data dir: {:?}", self.test_config.data_dir);

        let old_state = if self.test_config.save_state() {
            self.load_state_data().await.map(Arc::new)
        } else {
            None
        };

        ApiClient::new(self.test_config.api_urls.clone(), &self.reqwest_client).print_to_log();

        let (quit_now, server) = if !self.test_config.no_servers {
            info!("Creating ServerManager...");
            select! {
                result = signal::ctrl_c() =>
                    match result {
                        Ok(()) => (true, None),
                        Err(e) => {
                            error!("Failed to listen CTRL+C. Error: {}", e);
                            (true, None)
                        }
                    },
                server = ServerManager::new(&self.server_instance_config, self.test_config.clone(), None) => {
                    info!("...crated");
                    (false, Some(server))
                }
            }
        } else {
            (false, None)
        };

        let (bot_running_handle, mut wait_all_bots) = mpsc::channel::<BotPersistentState>(1);
        let (quit_handle, bot_quit_receiver) = watch::channel(());

        let mut task_number = self.test_config.tasks();

        if !quit_now {
            info!("Task count: {}", self.test_config.tasks());

            while task_number > 0 {
                let current_task_id = task_number - 1;

                // Check if this is admin bot task in bot mode
                if let Some(bot_mode) = self.test_config.bot_mode()
                    && bot_mode.admin
                    && current_task_id == 0
                {
                    let admin_bot = AdminBot::new(
                        current_task_id,
                        self.test_config.clone(),
                        self.bot_config_file.clone(),
                        old_state.clone(),
                        bot_running_handle.clone(),
                        &self.reqwest_client,
                    );
                    let quit_receiver = bot_quit_receiver.clone();
                    tokio::spawn(admin_bot.run(quit_receiver));
                } else if let Some(benchmark) = self.test_config.selected_benchmark() {
                    let benchmark_bot = BenchmarkBot::new(
                        current_task_id,
                        self.test_config.clone(),
                        self.bot_config_file.clone(),
                        *benchmark,
                        bot_running_handle.clone(),
                        &self.reqwest_client,
                    );
                    let quit_receiver = bot_quit_receiver.clone();
                    tokio::spawn(benchmark_bot.run(quit_receiver));
                } else {
                    let user_bot = UserBot::new(
                        current_task_id,
                        self.test_config.clone(),
                        self.bot_config_file.clone(),
                        old_state.clone(),
                        bot_running_handle.clone(),
                        &self.reqwest_client,
                    );
                    let quit_receiver = bot_quit_receiver.clone();
                    tokio::spawn(user_bot.run(quit_receiver));
                }

                task_number -= 1;
            }

            info!("Bot tasks are now created");
        }

        drop(bot_running_handle);
        drop(bot_quit_receiver);

        let mut bot_states = vec![];
        loop {
            select! {
                result = signal::ctrl_c() => {
                    match result {
                        Ok(()) => (),
                        Err(e) => error!("Failed to listen CTRL+C. Error: {}", e),
                    }
                    break
                }
                value = wait_all_bots.recv() => {
                    match value {
                        None => break,
                        Some(state) => bot_states.push(state),
                    }
                }
            }
        }

        drop(quit_handle); // Singnal quit to bots.

        // Wait that all bot_running_handles are dropped.
        loop {
            match wait_all_bots.recv().await {
                None => break,
                Some(state) => bot_states.push(state),
            }
        }

        let new_state = StateData {
            test_name: self.test_config.test_name(),
            bot_states,
        };

        if self.test_config.save_state() {
            let new_state = Self::merge_old_and_new_state_data(old_state.clone(), new_state);
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

    fn merge_old_and_new_state_data(old: Option<Arc<StateData>>, new: StateData) -> StateData {
        let mut bot_data: HashMap<u32, BotPersistentState> = HashMap::new();
        if let Some(old_state) = &old {
            for s in old_state.bot_states.iter().cloned() {
                bot_data.insert(s.task, s);
            }
        }
        for s in new.bot_states {
            bot_data.insert(s.task, s);
        }
        let mut data: Vec<BotPersistentState> = bot_data.into_values().collect();
        data.sort_by(|a, b| a.task.cmp(&b.task));

        StateData {
            test_name: new.test_name,
            bot_states: data,
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
        let data_file = format!("test_{}_state_data.json", self.test_config.test_name());
        DataDirUtils::create_data_dir_if_needed(&self.test_config).join(data_file)
    }
}
