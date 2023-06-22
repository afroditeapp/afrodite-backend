//! Bots for fake clients

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

pub struct ClientBot {
    state: BotState,
    actions: Peekable<Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>>,
}

impl Debug for ClientBot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ClientBot").finish()
    }
}

impl ClientBot {
    pub fn new(state: BotState) -> Self {
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

}

#[async_trait]
impl BotStruct for ClientBot {
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
        Ok(())
    }
}
