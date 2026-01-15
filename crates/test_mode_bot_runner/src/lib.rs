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

use api_client::apis::account_bot_api;
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

        if !quit_now {
            if let Some(benchmark) = self.test_config.selected_benchmark() {
                let benchmark = *benchmark;
                Self::spawn_benchmark_tasks(
                    self.test_config.clone(),
                    self.bot_config_file.clone(),
                    bot_running_handle.clone(),
                    bot_quit_receiver.clone(),
                    &self.reqwest_client,
                    benchmark,
                )
                .await;
            } else {
                Self::spawn_admin_and_user_bot_tasks(
                    self.test_config.clone(),
                    self.bot_config_file.clone(),
                    old_state.clone(),
                    bot_running_handle.clone(),
                    bot_quit_receiver.clone(),
                    &self.reqwest_client,
                )
                .await;
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

    async fn spawn_benchmark_tasks(
        test_config: Arc<TestMode>,
        bot_config_file: Arc<BotConfigFile>,
        bot_running_handle: mpsc::Sender<BotPersistentState>,
        bot_quit_receiver: watch::Receiver<()>,
        reqwest_client: &reqwest::Client,
        benchmark: config::args::SelectedBenchmark,
    ) {
        info!("Task count: {}", test_config.tasks());

        for task_id in 0..test_config.tasks() {
            let benchmark_bot = BenchmarkBot::new(
                task_id,
                test_config.clone(),
                bot_config_file.clone(),
                benchmark,
                bot_running_handle.clone(),
                reqwest_client,
            );
            let quit_receiver = bot_quit_receiver.clone();
            tokio::spawn(benchmark_bot.run(quit_receiver));
        }
    }

    async fn spawn_admin_and_user_bot_tasks(
        test_config: Arc<TestMode>,
        bot_config_file: Arc<BotConfigFile>,
        old_state: Option<Arc<StateData>>,
        bot_running_handle: mpsc::Sender<BotPersistentState>,
        bot_quit_receiver: watch::Receiver<()>,
        reqwest_client: &reqwest::Client,
    ) {
        let bot_accounts =
            Self::get_or_create_bot_accounts(test_config.clone(), reqwest_client).await;

        let has_admin = bot_accounts
            .admin
            .as_ref()
            .and_then(|b| b.as_ref())
            .is_some();
        let user_count = bot_accounts
            .users
            .as_ref()
            .map(|u| u.len() as u32)
            .unwrap_or(0);

        // Spawn admin bot first if needed (task_id 0)
        if has_admin {
            info!("Creating admin bot");

            let account_id_from_api = bot_accounts
                .admin
                .as_ref()
                .and_then(|b| b.as_ref())
                .map(|b| b.aid.as_ref().clone())
                .unwrap_or_else(|| api_client::models::AccountId::new(String::new()));

            let admin_bot = AdminBot::new(
                0,
                test_config.clone(),
                bot_config_file.clone(),
                old_state.clone(),
                bot_running_handle.clone(),
                account_id_from_api,
                reqwest_client,
            );
            let quit_receiver = bot_quit_receiver.clone();
            tokio::spawn(admin_bot.run(quit_receiver));
        }

        if user_count > 0 {
            info!("Creating {} user bots", user_count);
        }

        // Spawn user bots starting from task_id 1
        for (user_index, _) in (0..user_count).enumerate() {
            let task_id = 1 + user_index as u32;

            let account_id_from_api = bot_accounts
                .users
                .as_ref()
                .and_then(|users| users.get(user_index))
                .map(|b| b.aid.as_ref().clone())
                .unwrap_or_else(|| api_client::models::AccountId::new(String::new()));

            let user_bot = UserBot::new(
                task_id,
                test_config.clone(),
                bot_config_file.clone(),
                old_state.clone(),
                bot_running_handle.clone(),
                account_id_from_api,
                reqwest_client,
            );
            let quit_receiver = bot_quit_receiver.clone();
            tokio::spawn(user_bot.run(quit_receiver));
        }
    }

    async fn get_or_create_bot_accounts(
        test_config: Arc<TestMode>,
        reqwest_client: &reqwest::Client,
    ) -> api_client::models::GetBotsResult {
        let api_client = ApiClient::new(test_config.api_urls.clone(), reqwest_client);
        account_bot_api::post_get_bots(api_client.api())
            .await
            .expect("Failed to get bot accounts from server")
    }
}
