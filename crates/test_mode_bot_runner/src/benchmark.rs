//! Bots for benchmarking

use std::{
    fmt::Debug,
    iter::Peekable,
    sync::atomic::{AtomicU32, Ordering},
    time::{Duration, Instant},
};

use api_client::{
    apis::profile_api::{
        get_profile, get_profile_from_database_debug_mode_benchmark,
        post_profile_to_database_debug_mode_benchmark,
    },
    models::ProfileUpdate,
};
use async_trait::async_trait;
use error_stack::{Result, ResultExt};
use test_mode_bot::{
    BotState, BotStruct, TaskState, action_array,
    actions::{
        ActionArray, BotAction, RepeatUntilFn, RepeatUntilFnSimple, RunActions, RunFn, SleepMillis,
        TO_ADMIN_NORMAL_STATE, TO_NORMAL_STATE,
        account::{Login, Register, SetProfileVisibility},
        admin::content::ModerateContentModerationRequest,
        profile::{
            ChangeProfileText, GetProfileList, ProfileText, ResetProfileIterator,
            UpdateLocationRandomOrConfigured,
        },
    },
};
use test_mode_utils::{client::TestError, server::DEFAULT_LOCATION_CONFIG_BENCHMARK};
use tokio::time::sleep;
use tracing::log::info;

use crate::utils::Counters;

static COUNTERS: Counters = Counters::new();

pub struct Benchmark {
    state: BotState,
    actions: Peekable<Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>>,
}

impl Debug for Benchmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Benchmark").finish()
    }
}

impl Benchmark {
    pub fn benchmark_get_profile(state: BotState) -> Self {
        let setup = [&Register as &dyn BotAction, &Login];
        let benchmark = [
            &ActionsBeforeIteration as &dyn BotAction,
            &GetProfile,
            &ActionsAfterIteration,
        ];
        let iter = setup.into_iter().chain(benchmark.into_iter().cycle());
        Self {
            state,
            actions: (Box::new(iter)
                as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>)
                .peekable(),
        }
    }

    pub fn benchmark_get_profile_from_database(state: BotState) -> Self {
        let setup = [&Register as &dyn BotAction, &Login];
        let benchmark = [
            &ActionsBeforeIteration as &dyn BotAction,
            &GetProfileFromDatabase,
            &ActionsAfterIteration,
        ];
        let iter = setup.into_iter().chain(benchmark.into_iter().cycle());
        Self {
            state,
            actions: (Box::new(iter)
                as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>)
                .peekable(),
        }
    }

    pub fn benchmark_get_profile_list(state: BotState) -> Self {
        static READY_COUNT: AtomicU32 = AtomicU32::new(0);

        const SETUP: ActionArray = action_array![
            RunActions(TO_NORMAL_STATE),
            RunFn(|_| {
                READY_COUNT.fetch_add(1, Ordering::Relaxed);
            }),
            RepeatUntilFnSimple(
                |s| READY_COUNT.load(Ordering::Relaxed) == s.config.tasks() - 2,
                true,
                &SleepMillis(1)
            ),
        ];
        const BENCHMARK: ActionArray = action_array![
            ActionsBeforeIteration,
            ResetProfileIterator,
            RepeatUntilFn(|v, _| v.profile_count(), 0, &GetProfileListBenchmark),
            ActionsAfterIteration,
        ];
        let iter = SETUP
            .iter()
            .copied()
            .chain(BENCHMARK.iter().copied().cycle());
        Self {
            state,
            actions: (Box::new(iter)
                as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>)
                .peekable(),
        }
    }

    pub fn benchmark_get_profile_list_admin_bot(state: BotState) -> Self {
        const ACTIONS: ActionArray = action_array![
            RunActions(TO_ADMIN_NORMAL_STATE),
            ModerateContentModerationRequest::moderate_all_initial_content(),
        ];
        let iter = ACTIONS.iter().copied();
        Self {
            state,
            actions: (Box::new(iter)
                as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>)
                .peekable(),
        }
    }

    pub fn benchmark_get_profile_list_bot(state: BotState) -> Self {
        const ACTIONS: ActionArray = action_array![
            RunActions(TO_NORMAL_STATE),
            UpdateLocationRandomOrConfigured::new_deterministic(Some(
                DEFAULT_LOCATION_CONFIG_BENCHMARK
            )),
            SetProfileVisibility(true),
        ];
        let iter = ACTIONS.iter().copied();
        Self {
            state,
            actions: (Box::new(iter)
                as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>)
                .peekable(),
        }
    }

    pub fn benchmark_post_profile(state: BotState) -> Self {
        let setup = [&Register as &dyn BotAction, &Login];
        let benchmark = [
            &ActionsBeforeIteration as &dyn BotAction,
            &PostProfile,
            &ActionsAfterIteration,
        ];
        let iter = setup.into_iter().chain(benchmark.into_iter().cycle());
        Self {
            state,
            actions: (Box::new(iter)
                as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>)
                .peekable(),
        }
    }

    pub fn benchmark_post_profile_to_database(state: BotState) -> Self {
        let setup = [&Register as &dyn BotAction, &Login];
        let benchmark = [
            &ActionsBeforeIteration as &dyn BotAction,
            &PostProfileToDatabase,
            &ActionsAfterIteration,
        ];
        let iter = setup.into_iter().chain(benchmark.into_iter().cycle());
        Self {
            state,
            actions: (Box::new(iter)
                as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>)
                .peekable(),
        }
    }
}

#[async_trait]
impl BotStruct for Benchmark {
    fn peek_action_and_state(&mut self) -> (Option<&'static dyn BotAction>, &mut BotState) {
        (self.actions.peek().copied(), &mut self.state)
    }
    fn next_action(&mut self) {
        self.actions.next();
    }
    fn state(&self) -> &BotState {
        &self.state
    }
}

#[derive(Debug)]
pub struct GetProfile;

#[async_trait]
impl BotAction for GetProfile {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        get_profile(state.api(), &state.account_id_string()?, None, None)
            .await
            .change_context(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct GetProfileListBenchmark;

#[async_trait]
impl BotAction for GetProfileListBenchmark {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let result = GetProfileList.excecute_impl(state).await;
        COUNTERS.inc_sub();
        result
    }

    fn previous_value_supported(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct GetProfileFromDatabase;

#[async_trait]
impl BotAction for GetProfileFromDatabase {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        get_profile_from_database_debug_mode_benchmark(state.api(), &state.account_id_string()?)
            .await
            .change_context(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct PostProfile;

#[async_trait]
impl BotAction for PostProfile {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        ChangeProfileText {
            mode: ProfileText::Random,
        }
        .excecute(state, task_state)
        .await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct PostProfileToDatabase;

#[async_trait]
impl BotAction for PostProfileToDatabase {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        _task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        // Uuid has same string size every time.
        let profile = simple_backend_utils::UuidBase64Url::new_random_id();
        let profile = ProfileUpdate {
            attributes: vec![],
            age: 18,
            name: "Bot".to_string(),
            ptext: Some(profile.to_string()),
        };
        post_profile_to_database_debug_mode_benchmark(state.api(), profile)
            .await
            .change_context(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
struct ActionsBeforeIteration;

#[async_trait]
impl BotAction for ActionsBeforeIteration {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        if !state.config.no_sleep() {
            sleep(Duration::from_millis(1000)).await;
        }

        state.benchmark.action_duration = Instant::now();

        Ok(())
    }
}

#[derive(Debug)]
struct ActionsAfterIteration;

#[async_trait]
impl BotAction for ActionsAfterIteration {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        COUNTERS.inc_main();

        if state.print_info() {
            info!(
                "{:?}: {:?}, total: {}, details: {}",
                state.previous_action,
                state.benchmark.action_duration.elapsed(),
                COUNTERS.reset_main(),
                COUNTERS.reset_sub(),
            );
        }
        Ok(())
    }
}
