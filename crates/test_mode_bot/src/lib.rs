#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod actions;
pub mod benchmark;
pub mod connection;
pub mod utils;

use std::{
    fmt::Debug,
    sync::{Arc, atomic::Ordering},
    vec,
};

use actions::{admin::AdminBotState, chat::ChatState, profile::ProfileState};
use api_client::models::{AccountId, EventToClient};
use async_trait::async_trait;
use config::{
    Config,
    args::{PublicApiUrls, TestMode},
    bot_config_file::{BaseBotConfig, BotConfigFile},
};
use error_stack::{Result, ResultExt};
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use test_mode_utils::{
    client::{ApiClient, TestError},
    state::{BotEncryptionKeys, BotPersistentState},
};

use self::actions::{BotAction, DoNothing, PreviousValue, media::MediaState};
use crate::{benchmark::BenchmarkState, connection::BotConnections};

#[derive(Debug, Default)]
pub struct TaskState;

#[derive(Debug)]
pub struct BotState {
    pub id: Option<AccountId>,
    pub server_config: Arc<Config>,
    pub config: Arc<TestMode>,
    bot_config_file: Arc<BotConfigFile>,
    pub task_id: u32,
    pub bot_id: u32,
    pub api: ApiClient,
    pub api_urls: PublicApiUrls,
    pub previous_action: &'static dyn BotAction,
    pub previous_value: PreviousValue,
    pub action_history: Vec<&'static dyn BotAction>,
    pub benchmark: BenchmarkState,
    pub media: MediaState,
    pub profile: ProfileState,
    pub chat: ChatState,
    pub admin: AdminBotState,
    pub connections: BotConnections,
    pub refresh_token: Option<Vec<u8>>,
    pub deterministic_rng: Xoshiro256PlusPlus,
    pub reqwest_client: reqwest::Client,
}

impl BotState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Option<AccountId>,
        keys: Option<BotEncryptionKeys>,
        server_config: Arc<Config>,
        config: Arc<TestMode>,
        bot_config_file: Arc<BotConfigFile>,
        task_id: u32,
        bot_id: u32,
        api: ApiClient,
        api_urls: PublicApiUrls,
        reqwest_client: reqwest::Client,
    ) -> Self {
        Self {
            reqwest_client,
            id,
            server_config,
            config,
            bot_config_file,
            task_id,
            bot_id,
            api,
            api_urls,
            benchmark: BenchmarkState::new(),
            previous_action: &DoNothing,
            previous_value: PreviousValue::Empty,
            action_history: vec![],
            media: MediaState::new(),
            profile: ProfileState::new(),
            chat: ChatState { keys },
            admin: AdminBotState::default(),
            connections: BotConnections::default(),
            refresh_token: None,
            deterministic_rng: {
                let task_i_u64: u64 = task_id.into();
                let task_i_shifted = task_i_u64 << 32;
                let bot_i_u64: u64 = bot_id.into();
                Xoshiro256PlusPlus::seed_from_u64(task_i_shifted + bot_i_u64)
            },
        }
    }

    /// Wait event if event sending enabled or timeout after 5 seconds
    pub async fn wait_event(
        &mut self,
        check: impl Fn(&EventToClient) -> bool,
    ) -> Result<(), TestError> {
        self.connections.wait_event(check).await
    }

    pub fn are_events_enabled(&self) -> bool {
        self.connections
            .enable_event_sending
            .load(Ordering::Relaxed)
    }

    pub fn enable_events(&self) {
        self.connections
            .enable_event_sending
            .store(true, Ordering::Relaxed);
    }

    pub fn disable_events(&self) {
        self.connections
            .enable_event_sending
            .store(true, Ordering::Relaxed);
    }

    pub fn account_id(&self) -> Result<AccountId, TestError> {
        self.id.clone().ok_or(TestError::AccountIdMissing.report())
    }

    pub fn account_id_string(&self) -> Result<String, TestError> {
        self.id
            .clone()
            .ok_or(TestError::AccountIdMissing.report())
            .map(|id| id.aid)
    }

    pub fn is_first_bot(&self) -> bool {
        self.task_id == 0 && self.bot_id == 0
    }

    pub fn print_info(&mut self) -> bool {
        self.is_first_bot() && self.benchmark.print_info_timer.passed()
    }

    pub fn persistent_state(&self) -> Option<BotPersistentState> {
        self.id.clone().map(|id| BotPersistentState {
            account_id: id.aid,
            keys: self.chat.keys.clone(),
            task: self.task_id,
            bot: self.bot_id,
        })
    }

    /// Is current bot an bot mode admin bot.
    ///
    /// All bots in task ID 1 are admin bots in bot mode.
    pub fn is_bot_mode_admin_bot(&self) -> bool {
        self.config.bot_mode().is_some() && self.task_id == 1
    }

    /// Default [BaseBotConfig] is returned when current mode is other than
    /// [TestModeSubMode::Bot] even if the bot config file exists.
    pub fn get_bot_config(&self) -> &BaseBotConfig {
        self.bot_config_file
            .find_bot_config(self.bot_id)
            .map(|v| &v.config)
            .unwrap_or(&self.bot_config_file.bot_config)
    }

    pub fn remote_bot_password(&self) -> Option<String> {
        if self.config.bot_mode().is_some() {
            if self.is_bot_mode_admin_bot() {
                self.bot_config_file
                    .admin_bot_config
                    .remote_bot_login_password
                    .clone()
            } else {
                self.bot_config_file
                    .find_bot_config(self.bot_id)
                    .and_then(|v| v.remote_bot_login_password.clone())
            }
        } else {
            None
        }
    }
}

/// Bot completed
pub struct Completed;

#[async_trait]
pub trait BotStruct: Debug + Send + 'static {
    fn peek_action_and_state(&mut self) -> (Option<&'static dyn BotAction>, &mut BotState);
    fn next_action(&mut self);
    fn state(&self) -> &BotState;

    async fn run_action(
        &mut self,
        task_state: &mut TaskState,
    ) -> Result<Option<Completed>, TestError> {
        let mut result = self.run_action_impl(task_state).await;
        if self.state().config.qa_mode().is_some() {
            result = result.attach_printable_lazy(|| format!("{:?}", self.state().action_history))
        }
        result.attach_printable_lazy(|| format!("{__self:?}"))
    }

    async fn run_action_impl(
        &mut self,
        task_state: &mut TaskState,
    ) -> Result<Option<Completed>, TestError> {
        match self.peek_action_and_state() {
            (None, _) => Ok(Some(Completed)),
            (Some(action), state) => {
                let result = action.excecute(state, task_state).await;

                let result = match result {
                    Err(e) if e.current_context() == &TestError::BotIsWaiting => return Ok(None),
                    Err(e) => Err(e),
                    Ok(()) => Ok(None),
                };

                state.previous_action = action;
                if state.config.qa_mode().is_some() {
                    state.action_history.push(action)
                }
                self.next_action();
                result
            }
        }
    }
}
