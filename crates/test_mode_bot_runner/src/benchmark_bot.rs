use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use config::{
    args::{SelectedBenchmark, TestMode},
    bot_config_file::BotConfigFile,
};
use error_stack::Result;
use test_mode_bot::{
    BotState,
    actions::{
        BotAction, RunActions, TO_NORMAL_STATE, account::SetProfileVisibility,
        profile::UpdateLocationRandomOrConfigured,
    },
};
use test_mode_utils::{
    client::{ApiClient, TestError},
    server::DEFAULT_LOCATION_CONFIG_BENCHMARK,
    state::BotPersistentState,
};
use tokio::{
    select,
    sync::{mpsc, watch},
};
use tracing::error;

static BENCHMARK_GET_PROFILE_LIST_INDEX_READY: AtomicBool = AtomicBool::new(false);

pub struct BenchmarkBot {
    state: BotState,
    benchmark_type: SelectedBenchmark,
    task_id: u32,
    bot_running_handle: mpsc::Sender<BotPersistentState>,
}

impl BenchmarkBot {
    pub fn new(
        task_id: u32,
        config: Arc<TestMode>,
        bot_config_file: Arc<BotConfigFile>,
        benchmark_type: SelectedBenchmark,
        bot_running_handle: mpsc::Sender<BotPersistentState>,
        reqwest_client: &reqwest::Client,
    ) -> Self {
        let mut state = BotState::new(
            None,
            None,
            config.clone(),
            bot_config_file.clone(),
            task_id,
            ApiClient::new(config.api_urls.clone(), reqwest_client),
            config.api_urls.clone(),
            reqwest_client.clone(),
        );

        if (benchmark_type == SelectedBenchmark::GetProfileList && task_id == 1)
            || (benchmark_type != SelectedBenchmark::GetProfileList && task_id == 0)
        {
            state.benchmark.print_benchmark_info = true;
        }

        Self {
            state,
            benchmark_type,
            task_id,
            bot_running_handle,
        }
    }

    async fn handle_quit(
        persistent_state: Option<BotPersistentState>,
        bot_running_handle: mpsc::Sender<BotPersistentState>,
    ) {
        if let Some(persistent_state) = persistent_state
            && let Err(e) = bot_running_handle.send(persistent_state).await
        {
            error!("Failed to send benchmark bot state: {:?}", e);
        }
    }

    pub async fn run(mut self, mut bot_quit_receiver: watch::Receiver<()>) {
        select! {
            result = Self::run_benchmark_loop(&mut self.state, self.benchmark_type, self.task_id) => {
                match result {
                    Ok(()) => (),
                    Err(e) => {
                        error!("Benchmark bot error - Task {}: {:?}", self.task_id, e);
                    }
                }
            },
            _ = bot_quit_receiver.changed() => {}
        };

        Self::handle_quit(self.state.persistent_state(), self.bot_running_handle).await;
    }

    async fn run_benchmark_loop(
        state: &mut BotState,
        benchmark_type: SelectedBenchmark,
        task_id: u32,
    ) -> Result<(), TestError> {
        match benchmark_type {
            SelectedBenchmark::GetProfile => Self::benchmark_get_profile(state).await,
            SelectedBenchmark::GetProfileFromDatabase => {
                Self::benchmark_get_profile_from_database(state).await
            }
            SelectedBenchmark::GetProfileList => {
                if task_id == 0 {
                    Self::benchmark_get_profile_list_setup_task(state).await
                } else {
                    Self::benchmark_get_profile_list(state).await
                }
            }
            SelectedBenchmark::PostProfile => Self::benchmark_post_profile(state).await,
            SelectedBenchmark::PostProfileToDatabase => {
                Self::benchmark_post_profile_to_database(state).await
            }
        }
    }

    async fn benchmark_get_profile(state: &mut BotState) -> Result<(), TestError> {
        use test_mode_bot::actions::account::{Login, Register};

        use crate::actions::benchmark::{
            ActionsAfterIteration, ActionsBeforeIteration, GetProfile,
        };

        // Setup
        Register.excecute(state).await?;
        Login.excecute(state).await?;

        // Benchmark loop
        loop {
            ActionsBeforeIteration.excecute(state).await?;
            GetProfile.excecute(state).await?;
            ActionsAfterIteration.excecute(state).await?;
        }
    }

    async fn benchmark_get_profile_from_database(state: &mut BotState) -> Result<(), TestError> {
        use test_mode_bot::actions::account::{Login, Register};

        use crate::actions::benchmark::{
            ActionsAfterIteration, ActionsBeforeIteration, GetProfileFromDatabase,
        };

        // Setup
        Register.excecute(state).await?;
        Login.excecute(state).await?;

        // Benchmark loop
        loop {
            ActionsBeforeIteration.excecute(state).await?;
            GetProfileFromDatabase.excecute(state).await?;
            ActionsAfterIteration.excecute(state).await?;
        }
    }

    async fn benchmark_get_profile_list(state: &mut BotState) -> Result<(), TestError> {
        use std::sync::atomic::{AtomicU32, Ordering};

        use test_mode_bot::{
            action_array,
            actions::{
                ActionArray, RepeatUntilFn, RepeatUntilFnSimple, RunActions, RunFn, SleepMillis,
                TO_NORMAL_STATE, profile::ResetProfileIterator,
            },
        };

        use crate::actions::benchmark::{
            ActionsAfterIteration, ActionsBeforeIteration, GetProfileListBenchmark,
        };

        loop {
            if BENCHMARK_GET_PROFILE_LIST_INDEX_READY.load(Ordering::Relaxed) {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await
        }

        static READY_COUNT: AtomicU32 = AtomicU32::new(0);

        // Setup
        const SETUP: ActionArray = action_array![
            RunActions(TO_NORMAL_STATE),
            RunFn(|_| {
                READY_COUNT.fetch_add(1, Ordering::Relaxed);
            }),
            RepeatUntilFnSimple(
                |s| READY_COUNT.load(Ordering::Relaxed) == s.config.tasks() - 1,
                true,
                &SleepMillis(1)
            ),
        ];

        for action in SETUP.iter() {
            action.excecute(state).await?;
        }

        // Benchmark loop
        loop {
            ActionsBeforeIteration.excecute(state).await?;
            ResetProfileIterator.excecute(state).await?;
            RepeatUntilFn(|v, _| v.profile_count(), 0, &GetProfileListBenchmark)
                .excecute(state)
                .await?;
            ActionsAfterIteration.excecute(state).await?;
        }
    }

    async fn benchmark_get_profile_list_setup_task(state: &mut BotState) -> Result<(), TestError> {
        use test_mode_bot::actions::{
            RunActions, TO_ADMIN_NORMAL_STATE, admin::content::ModerateContentModerationRequest,
        };

        RunActions(TO_ADMIN_NORMAL_STATE).excecute(state).await?;
        UpdateLocationRandomOrConfigured::new_deterministic(Some(
            DEFAULT_LOCATION_CONFIG_BENCHMARK,
        ))
        .excecute(state)
        .await?;
        SetProfileVisibility(true).excecute(state).await?;
        ModerateContentModerationRequest::moderate_all_initial_content()
            .excecute(state)
            .await?;

        BENCHMARK_GET_PROFILE_LIST_INDEX_READY.store(true, Ordering::Relaxed);

        Ok(())
    }

    async fn benchmark_post_profile(state: &mut BotState) -> Result<(), TestError> {
        use crate::actions::benchmark::{
            ActionsAfterIteration, ActionsBeforeIteration, PostProfile,
        };

        // Setup
        RunActions(TO_NORMAL_STATE).excecute(state).await?;

        // Benchmark loop
        loop {
            ActionsBeforeIteration.excecute(state).await?;
            PostProfile.excecute(state).await?;
            ActionsAfterIteration.excecute(state).await?;
        }
    }

    async fn benchmark_post_profile_to_database(state: &mut BotState) -> Result<(), TestError> {
        use test_mode_bot::actions::account::{Login, Register};

        use crate::actions::benchmark::{
            ActionsAfterIteration, ActionsBeforeIteration, PostProfileToDatabase,
        };

        // Setup
        Register.excecute(state).await?;
        Login.excecute(state).await?;

        // Benchmark loop
        loop {
            ActionsBeforeIteration.excecute(state).await?;
            PostProfileToDatabase.excecute(state).await?;
            ActionsAfterIteration.excecute(state).await?;
        }
    }
}
