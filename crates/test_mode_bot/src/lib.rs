#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod actions;
pub mod benchmark;
pub mod connection;
pub mod utils;

use std::{fmt::Debug, sync::Arc};

use actions::{chat::ChatState, profile::ProfileState};
use api_client::{
    apis::configuration::Configuration,
    models::{AccountId, EventToClient},
};
use config::{
    args::{PublicApiUrl, TestMode},
    bot_config_file::{BaseBotConfig, BotConfigFile},
};
use error_stack::Result;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use test_mode_utils::{
    client::{ApiClient, TestError},
    state::{BotEncryptionKeys, BotPersistentState},
};

use self::actions::{PreviousValue, media::MediaState};
use crate::{benchmark::BenchmarkState, connection::BotConnections};

#[derive(Debug)]
pub struct BotState {
    pub id: Option<AccountId>,
    pub config: Arc<TestMode>,
    pub bot_config_file: Arc<BotConfigFile>,
    pub task_id: u32,
    pub api: ApiClient,
    pub api_urls: PublicApiUrl,
    pub previous_value: PreviousValue,
    pub benchmark: BenchmarkState,
    pub media: MediaState,
    pub profile: ProfileState,
    pub chat: ChatState,
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
        config: Arc<TestMode>,
        bot_config_file: Arc<BotConfigFile>,
        task_id: u32,
        api: ApiClient,
        api_urls: PublicApiUrl,
        reqwest_client: reqwest::Client,
    ) -> Self {
        Self {
            reqwest_client,
            id,
            config,
            bot_config_file,
            task_id,
            api,
            api_urls,
            benchmark: BenchmarkState::new(),
            previous_value: PreviousValue::Empty,
            media: MediaState::new(),
            profile: ProfileState::new(),
            chat: ChatState { keys },
            connections: BotConnections::default(),
            refresh_token: None,
            deterministic_rng: {
                let task_i_u64: u64 = task_id.into();
                Xoshiro256PlusPlus::seed_from_u64(task_i_u64)
            },
        }
    }

    pub fn api(&self) -> &Configuration {
        self.api.api()
    }

    /// Wait event if event sending enabled or timeout after 5 seconds
    pub async fn wait_event(
        &mut self,
        check: impl Fn(&EventToClient) -> bool,
    ) -> Result<(), TestError> {
        self.connections.wait_event(check).await
    }

    pub fn are_events_enabled(&self) -> bool {
        self.connections.are_events_enabled()
    }

    pub fn enable_events(&self) {
        self.connections.enable_events();
    }

    pub fn disable_events(&self) {
        self.connections.disable_events();
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

    pub fn persistent_state(&self) -> Option<BotPersistentState> {
        self.id.clone().map(|id| BotPersistentState {
            account_id: id.aid,
            keys: self.chat.keys.clone(),
            task: self.task_id,
        })
    }

    /// Is current bot an bot mode admin bot.
    ///
    /// Task ID 0 bot is admin bot.
    pub fn is_bot_mode_admin_bot(&self) -> bool {
        self.config.bot_mode().is_some() && self.task_id == 0
    }

    /// Default [BaseBotConfig] is returned when current mode is other than
    /// [TestModeSubMode::Bot] even if the bot config file exists.
    pub fn get_bot_config(&self) -> &BaseBotConfig {
        self.bot_config_file
            .find_bot_config(self.task_id)
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
                    .find_bot_config(self.task_id)
                    .and_then(|v| v.remote_bot_login_password.clone())
            }
        } else {
            None
        }
    }
}

/// Bot completed
pub struct Completed;
