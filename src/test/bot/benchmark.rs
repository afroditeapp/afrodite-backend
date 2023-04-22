//! Bots for benchmarking

use std::{fmt::Debug, time::{Duration, Instant}};

use api_client::apis::profile_api::{get_profile, get_default_profile};
use async_trait::async_trait;
use tokio::time::sleep;

use crate::test::client::TestError;

use super::{BotState, BotStruct, actions::{BotAction, Register, Login, ChangeProfileText}, Completed, utils::{Timer, Counters}};


use error_stack::{Result, FutureExt, ResultExt};

use tracing::{error, log::warn, log::info};

use super::super::client::{ApiClient};

use crate::{
    api::model::AccountId,
    config::args::{Test, TestMode},
    utils::IntoReportExt,
};

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
    actions: Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>,
}

impl Debug for Benchmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Benchmark").finish()
    }
}

impl Benchmark {
    pub fn get_profile_benchmark(state: BotState) -> Self {
        let setup = [
            &Register as &dyn BotAction,
            &Login,
        ];
        let benchmark = [
            &UpdateProfileBenchmark as &dyn BotAction,
            &ActionsBeforeIteration,
            &GetProfile,
            &ActionsAfterIteration
        ];
        let iter = setup
            .into_iter()
            .chain(benchmark.into_iter().cycle());
        Self {
            state,
            actions: Box::new(iter),
        }
    }

    pub fn get_default_profile_benchmark(state: BotState) -> Self {
        let setup = [
            &Register as &dyn BotAction,
            &Login,
        ];
        let benchmark = [
            &UpdateProfileBenchmark as &dyn BotAction,
            &ActionsBeforeIteration,
            &GetDefaultProfile,
            &ActionsAfterIteration
        ];
        let iter = setup
            .into_iter()
            .chain(benchmark.into_iter().cycle());
        Self {
            state,
            actions: Box::new(iter),
        }
    }
}

#[async_trait]
impl BotStruct for Benchmark {
    fn next_action_and_state(&mut self) -> (Option<&'static dyn BotAction>, &mut BotState) {
        (self.actions.next(), &mut self.state)
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
pub struct GetDefaultProfile;

#[async_trait]
impl BotAction for GetDefaultProfile {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        get_default_profile(state.api.profile(), &state.id_string()?)
            .await
            .into_error(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateProfileBenchmark;

#[async_trait]
impl BotAction for UpdateProfileBenchmark {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let time = Instant::now();

        if state.config.update_profile && state.benchmark.update_profile_timer.passed() {
            ChangeProfileText.excecute(state).await?;

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
