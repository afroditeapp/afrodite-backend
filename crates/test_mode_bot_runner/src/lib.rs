#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! Bot mode related test/bot runner.

mod benchmark;
mod client_bot;
mod manager;
mod utils;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};

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

use crate::manager::BotManager;

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
        let start_cmd = std::env::args().next().unwrap();
        let start_cmd = std::fs::canonicalize(&start_cmd).unwrap();
        info!("Path to server binary: {:?}", &start_cmd);

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

        let (bot_running_handle, mut wait_all_bots) = mpsc::channel::<Vec<BotPersistentState>>(1);
        let (quit_handle, bot_quit_receiver) = watch::channel(());

        let mut task_number = self.test_config.tasks();

        if !quit_now {
            self.log_task_and_bot_count_info();

            while task_number > 0 {
                BotManager::spawn(
                    task_number - 1,
                    self.test_config.clone(),
                    self.bot_config_file.clone(),
                    old_state.clone(),
                    bot_quit_receiver.clone(),
                    bot_running_handle.clone(),
                    &self.reqwest_client,
                );

                // Special case for profile iterator benchmark:
                // wait until profile index bot profiles are creates and
                // then wait that images for those profiles are moderated.
                if self.test_config.selected_benchmark()
                    == Some(&config::args::SelectedBenchmark::GetProfileList)
                    && task_number >= self.test_config.tasks() - 1
                {
                    select! {
                        result = signal::ctrl_c() => {
                            match result {
                                Ok(()) => (),
                                Err(e) => error!("Failed to listen CTRL+C. Error: {}", e),
                            }
                            break
                        }
                        _ = wait_all_bots.recv() => (),
                    }
                }

                task_number -= 1;
            }

            info!("Bot tasks are now created",);
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
                        Some(states) => bot_states.extend(states),
                    }
                }
            }
        }

        drop(quit_handle); // Singnal quit to bots.

        // Wait that all bot_running_handles are dropped.
        loop {
            match wait_all_bots.recv().await {
                None => break,
                Some(states) => bot_states.extend(states),
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

    fn log_task_and_bot_count_info(&self) {
        let mut bot_counts: Vec<u32> = vec![];
        for task_id in 0..self.test_config.tasks() {
            bot_counts.push(self.test_config.bots(task_id));
        }
        let all_values_equal_info: HashSet<u32> = bot_counts.iter().copied().collect();
        if all_values_equal_info.len() <= 1 {
            info!(
                "Task count: {}, Bot count per task: {}",
                self.test_config.tasks(),
                self.test_config.bots(0),
            );
        } else {
            info!(
                "Task count: {}, Bot counts per task: {:?}",
                self.test_config.tasks(),
                bot_counts,
            );
        }
    }

    fn merge_old_and_new_state_data(old: Option<Arc<StateData>>, new: StateData) -> StateData {
        let mut bot_data: HashMap<(u32, u32), BotPersistentState> = HashMap::new();
        if let Some(old_state) = &old {
            for s in old_state.bot_states.iter().cloned() {
                bot_data.insert((s.task, s.bot), s);
            }
        }
        for s in new.bot_states {
            bot_data.insert((s.task, s.bot), s);
        }
        let mut data: Vec<BotPersistentState> = bot_data.into_values().collect();
        data.sort_by(|a, b| (a.task, a.bot).cmp(&(b.task, b.bot)));

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
