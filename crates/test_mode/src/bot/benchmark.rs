//! Bots for benchmarking

use std::{
    fmt::Debug,
    iter::Peekable,
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
use tokio::time::sleep;
use tracing::log::info;

use super::{
    actions::{
        account::{Login, Register, SetProfileVisibility},
        profile::{
            ChangeProfileText, GetProfileList, ProfileText, ResetProfileIterator,
            UpdateLocationRandom,
        },
        BotAction, RepeatUntilFn, RunActions, TO_NORMAL_STATE,
    },
    utils::{Counters, Timer},
    BotState, BotStruct, TaskState,
};
use crate::{client::TestError, server::DEFAULT_LOCATION_CONFIG_BENCHMARK};

static COUNTERS: Counters = Counters::new();

#[derive(Debug)]
pub struct BenchmarkState {
    pub update_profile_timer: Timer,
    pub print_info_timer: Timer,
    pub action_duration: Instant,
}

impl BenchmarkState {
    pub fn new() -> Self {
        Self {
            update_profile_timer: Timer::new(Duration::from_millis(1000)),
            print_info_timer: Timer::new(Duration::from_millis(1000)),
            action_duration: Instant::now(),
        }
    }
}

impl Default for BenchmarkState {
    fn default() -> Self {
        Self::new()
    }
}

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
        let setup = [&RunActions(TO_NORMAL_STATE) as &dyn BotAction];
        let benchmark = [
            &ActionsBeforeIteration as &dyn BotAction,
            &ResetProfileIterator,
            &RepeatUntilFn(|v, _| v.profile_count(), 0, &GetProfileList),
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

    pub fn benchmark_get_profile_list_bot(state: BotState) -> Self {
        let benchmark = [
            &RunActions(TO_NORMAL_STATE) as &dyn BotAction,
            &UpdateLocationRandom(Some(DEFAULT_LOCATION_CONFIG_BENCHMARK)),
            &SetProfileVisibility(true),
        ];
        let iter = benchmark.into_iter();
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
        get_profile(state.api.profile(), &state.account_id_string()?)
            .await
            .change_context(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct GetProfileFromDatabase;

#[async_trait]
impl BotAction for GetProfileFromDatabase {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        get_profile_from_database_debug_mode_benchmark(
            state.api.profile(),
            &state.account_id_string()?,
        )
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
        let profile = uuid::Uuid::new_v4(); // Uuid has same string size every time.
        let profile = ProfileUpdate {
            attributes: vec![],
            age: 18,
            name: String::new(),
            profile_text: format!("{}", profile),
        };
        post_profile_to_database_debug_mode_benchmark(state.api.profile(), profile)
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
        COUNTERS.inc_get_profile();

        if state.print_info() {
            info!(
                "{:?}: {:?}, total: {}",
                state.previous_action,
                state.benchmark.action_duration.elapsed(),
                COUNTERS.reset_get_profile()
            );
        }
        Ok(())
    }
}
