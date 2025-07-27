use std::{sync::Arc, vec};

use api_client::models::AccountId;
use config::{
    Config,
    args::{SelectedBenchmark, TestMode, TestModeSubMode},
    bot_config_file::BotConfigFile,
};
use test_mode_bot::{BotState, BotStruct, Completed, TaskState};
use test_mode_utils::{
    client::ApiClient,
    state::{BotPersistentState, StateData},
};
use tokio::{
    select,
    sync::{mpsc, watch},
};
use tracing::{error, info};

use crate::{benchmark::Benchmark, client_bot::ClientBot};

pub struct BotManager {
    bots: Vec<Box<dyn BotStruct>>,
    removed_bots: Vec<Box<dyn BotStruct>>,
    bot_running_handle: mpsc::Sender<Vec<BotPersistentState>>,
    task_id: u32,
    config: Arc<TestMode>,
}

impl BotManager {
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        task_id: u32,
        server_config: Arc<Config>,
        config: Arc<TestMode>,
        bot_config_file: Arc<BotConfigFile>,
        old_state: Option<Arc<StateData>>,
        bot_quit_receiver: watch::Receiver<()>,
        bot_running_handle: mpsc::Sender<Vec<BotPersistentState>>,
        reqwest_client: &reqwest::Client,
    ) {
        let bot = match config.mode {
            TestModeSubMode::Benchmark(_) | TestModeSubMode::Bot(_) => Self::benchmark_or_bot(
                task_id,
                old_state,
                server_config,
                bot_config_file,
                config,
                bot_running_handle,
                reqwest_client,
            ),
            TestModeSubMode::Qa(_) => panic!("Server tests use different test runner"),
        };

        tokio::spawn(bot.run(bot_quit_receiver));
    }

    pub fn benchmark_or_bot(
        task_id: u32,
        old_state: Option<Arc<StateData>>,
        server_config: Arc<Config>,
        bot_config_file: Arc<BotConfigFile>,
        config: Arc<TestMode>,
        bot_running_handle: mpsc::Sender<Vec<BotPersistentState>>,
        reqwest_client: &reqwest::Client,
    ) -> Self {
        let mut bots = Vec::<Box<dyn BotStruct>>::new();
        for bot_i in 0..config.bots(task_id) {
            let account_id = if config.bot_mode().is_some() {
                if task_id == 1 {
                    bot_config_file.admin_bot_config.account_id.clone()
                } else {
                    bot_config_file
                        .find_bot_config(bot_i)
                        .and_then(|v| v.account_id.clone())
                }
            } else {
                None
            };
            let account_id = account_id.or_else(|| {
                old_state
                    .as_ref()
                    .and_then(|v| v.find_matching(task_id, bot_i))
                    .map(|v| v.account_id.clone())
            });
            let keys = old_state
                .as_ref()
                .and_then(|v| v.find_matching(task_id, bot_i))
                .and_then(|v| v.keys.clone());
            let state = BotState::new(
                account_id.map(AccountId::new),
                keys,
                server_config.clone(),
                config.clone(),
                bot_config_file.clone(),
                task_id,
                bot_i,
                ApiClient::new(config.api_urls.clone(), reqwest_client),
                config.api_urls.clone(),
                reqwest_client.clone(),
            );

            match (config.selected_benchmark(), config.bot_mode()) {
                (Some(benchmark), _) => match benchmark {
                    SelectedBenchmark::GetProfile => {
                        bots.push(Box::new(Benchmark::benchmark_get_profile(state)))
                    }
                    SelectedBenchmark::GetProfileFromDatabase => bots.push(Box::new(
                        Benchmark::benchmark_get_profile_from_database(state),
                    )),
                    SelectedBenchmark::GetProfileList => {
                        let benchmark = if task_id == config.tasks() - 1 {
                            // Second last task is bot task
                            Benchmark::benchmark_get_profile_list_bot(state)
                        } else if task_id == config.tasks() - 2 {
                            // Last task is admin bot task
                            if bot_i == 0 {
                                Benchmark::benchmark_get_profile_list_admin_bot(state)
                            } else {
                                continue;
                            }
                        } else if bot_i == 0 {
                            // Create bot for benchmark task
                            Benchmark::benchmark_get_profile_list(state)
                        } else {
                            // Create only one benchmark bot per benchmark task.
                            continue;
                        };
                        bots.push(Box::new(benchmark))
                    }
                    SelectedBenchmark::PostProfile => {
                        bots.push(Box::new(Benchmark::benchmark_post_profile(state)))
                    }
                    SelectedBenchmark::PostProfileToDatabase => bots.push(Box::new(
                        Benchmark::benchmark_post_profile_to_database(state),
                    )),
                },
                (_, Some(_)) => bots.push(Box::new(ClientBot::new(state))),
                test_config => panic!("Invalid test config {test_config:?}"),
            };
        }

        Self {
            bots,
            removed_bots: vec![],
            bot_running_handle,
            task_id,
            config,
        }
    }

    pub async fn run(mut self, mut bot_quit_receiver: watch::Receiver<()>) {
        loop {
            select! {
                result = bot_quit_receiver.changed() => {
                    if result.is_err() {
                        break
                    }
                }
                _ = self.run_bot() => {
                    break;
                }
            }
        }

        let data = self.persistent_state_for_all_bots();
        self.bot_running_handle.send(data).await.unwrap();
    }

    fn persistent_state_for_all_bots(&self) -> Vec<BotPersistentState> {
        self.bots
            .iter()
            .filter_map(|bot| bot.state().persistent_state())
            .chain(
                self.removed_bots
                    .iter()
                    .filter_map(|bot| bot.state().persistent_state()),
            )
            .collect()
    }

    async fn run_bot(&mut self) {
        let mut errors = false;
        let mut task_state: TaskState = TaskState;
        loop {
            if self.config.early_quit && errors {
                error!("Error occurred in task {}", self.task_id);
                return;
            }

            if self.bots.is_empty() {
                if errors {
                    error!(
                        "All bots closed from task {}. Errors occurred.",
                        self.task_id
                    );
                } else {
                    info!("All bots closed from task {}. No errors.", self.task_id);
                }
                return;
            }

            if let Some(remove_i) = self.iter_bot_list(&mut errors, &mut task_state).await {
                self.removed_bots.push(self.bots.swap_remove(remove_i));
            }

            if let Some(bot_mode_config) = self.config.bot_mode() {
                if !bot_mode_config.no_sleep {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// If Some(bot_index) is returned remove the bot.
    async fn iter_bot_list(
        &mut self,
        errors: &mut bool,
        task_state: &mut TaskState,
    ) -> Option<usize> {
        for (i, b) in self.bots.iter_mut().enumerate() {
            match b.run_action(task_state).await {
                Ok(None) => (),
                Ok(Some(Completed)) => return Some(i),
                Err(e) => {
                    error!("Task {}, bot returned error: {:?}", self.task_id, e);
                    *errors = true;
                    return Some(i);
                }
            }
        }
        None
    }
}
