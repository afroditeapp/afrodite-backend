//! Bots for benchmarking

use std::{
    fmt::Debug,
    iter::Peekable,
    time::{Duration, Instant},
};

use api_client::apis::profile_api::get_profile;
use async_trait::async_trait;
use tokio::time::sleep;

use crate::{
    test::{client::TestError, server::DEFAULT_LOCATION_CONFIG_BENCHMARK},
};

use super::{
    actions::{
        account::{Login, Register, SetProfileVisibility},
        profile::{
            ChangeProfileText, GetProfileList, ResetProfileIterator,
            UpdateLocationRandom,
        },
        BotAction, RepeatUntilFn, RunActions, TO_NORMAL_STATE,
    },
    utils::{Counters, Timer},
    BotState, BotStruct, TaskState,
};

use error_stack::Result;

use tracing::log::info;

use crate::utils::IntoReportExt;

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
            &UpdateProfileBenchmark as &dyn BotAction,
            &ActionsBeforeIteration,
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

    pub fn benchmark_get_profile_list(state: BotState) -> Self {
        let setup = [&RunActions(TO_NORMAL_STATE) as &dyn BotAction];
        let benchmark = [
            &UpdateProfileBenchmark as &dyn BotAction,
            &ActionsBeforeIteration,
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
            &UpdateLocationRandom(DEFAULT_LOCATION_CONFIG_BENCHMARK),
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
        get_profile(state.api.profile(), &state.id_string()?)
            .await
            .into_error(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateProfileBenchmark;

#[async_trait]
impl BotAction for UpdateProfileBenchmark {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        let time = Instant::now();

        if state.config.update_profile && state.benchmark.update_profile_timer.passed() {
            ChangeProfileText.excecute(state, task_state).await?;

            if state.is_first_bot() {
                info!("post_profile: {:?}", time.elapsed());
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ActionsBeforeIteration;

#[async_trait]
impl BotAction for ActionsBeforeIteration {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        if !state.config.no_sleep {
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
