//! Bots for fake clients

use std::{
    fmt::Debug,
    iter::Peekable,
    time::{Duration, Instant},
};

use api_client::{
    apis::{account_api::get_account_state, profile_api::get_profile},
    models::AccountState,
};
use async_trait::async_trait;
use error_stack::{Result, ResultExt};
use tokio::time::sleep;

use super::{
    actions::{
        account::{AssertAccountState, Login, Register, SetAccountSetup, SetProfileVisibility},
        media::SendImageToSlot,
        BotAction, RunActions, profile::{UpdateLocation, UpdateLocationRandom, GetProfile}, RunActionsIf,
    },
    BotState, BotStruct, TaskState,
};
use crate::{
    action_array,
    bot::actions::{
        account::CompleteAccountSetup, admin::ModerateMediaModerationRequest,
        media::MakeModerationRequest, ActionArray,
    },
    client::TestError,
};

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
        let admin_bot = state
            .config
            .bot_mode()
            .map(|bot_config| state.bot_id < bot_config.admins)
            .unwrap_or(false);

        let iter = if admin_bot {
            // Admin bot

            let setup = [
                &Register as &dyn BotAction,
                &Login,
                &DoInitialSetupIfNeeded { admin: true },
            ];
            let action_loop = [
                &ActionsBeforeIteration as &dyn BotAction,
                &ModerateMediaModerationRequest,
                &ActionsAfterIteration,
            ];
            let iter = setup.into_iter().chain(action_loop.into_iter().cycle());

            Box::new(iter) as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>
        } else {
            // User bot

            let setup = [
                &Register as &dyn BotAction,
                &Login,
                &DoInitialSetupIfNeeded { admin: false },
                &UpdateLocationRandom(None),
                &SetProfileVisibility(true),
            ];
            let action_loop = [
                &ActionsBeforeIteration as &dyn BotAction,
                &GetProfile,
                &RunActionsIf(
                    action_array!(UpdateLocationRandom(None)),
                    || rand::random::<f32>() < 0.2,
                ),
                // TODO: Toggle the profile visiblity in the future?
                &RunActionsIf(
                    action_array!(SetProfileVisibility(true)),
                    || rand::random::<f32>() < 0.5,
                ),
                &RunActionsIf(
                    action_array!(SetProfileVisibility(false)),
                    || rand::random::<f32>() < 0.1,
                ),
                &ActionsAfterIteration,
            ];
            let iter = setup.into_iter().chain(action_loop.into_iter().cycle());

            Box::new(iter) as Box<dyn Iterator<Item = &'static dyn BotAction> + Send + Sync>
        };

        Self {
            state,
            actions: iter.peekable(),
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
pub struct DoInitialSetupIfNeeded {
    admin: bool,
}

#[async_trait]
impl BotAction for DoInitialSetupIfNeeded {
    async fn excecute_impl_task_state(
        &self,
        state: &mut BotState,
        task_state: &mut TaskState,
    ) -> Result<(), TestError> {
        let account_state = get_account_state(state.api.account())
            .await
            .change_context(TestError::ApiRequest)?;

        if account_state.state == AccountState::InitialSetup {
            if self.admin {
                SetAccountSetup::admin()
            } else {
                SetAccountSetup::new()
            }
            .excecute_impl_task_state(state, task_state)
            .await?;

            const ACTIONS: ActionArray = action_array!(
                SendImageToSlot {
                    slot: 1,
                    random: true,
                    copy_to_slot: Some(0),
                    mark_copied_image: true,
                },
                MakeModerationRequest { camera: true },
                CompleteAccountSetup,
                AssertAccountState(AccountState::Normal),
            );
            RunActions(ACTIONS)
                .excecute_impl_task_state(state, task_state)
                .await?;
        }

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
    async fn excecute_impl(&self, _state: &mut BotState) -> Result<(), TestError> {
        Ok(())
    }
}
