use std::{mem, sync::Arc};

use api_client::{
    apis::{
        configuration::Configuration,
        media_admin_api,
        profile_api::{post_profile, post_search_age_range, post_search_groups},
    },
    models::{
        AccountId, EventToClient, ModerationQueueType, ProfileSearchAgeRange, ProfileUpdate,
        SearchGroups,
    },
};
use config::{args::TestMode, bot_config_file::BotConfigFile, Config};
use error_stack::{Result, ResultExt};
use tokio::sync::Mutex;

use crate::{
    action_array,
    bot::{
        actions::{
            account::{CompleteAccountSetup, Login, Register, SetAccountSetup},
            admin::ModerateMediaModerationRequest,
            media::{MakeModerationRequest, SendImageToSlot, SetPendingContent},
            BotAction,
        },
        AccountConnections, BotState,
    },
    client::ApiClient,
    TestError,
};

#[derive(Debug)]
struct State {
    pub connections: Vec<AccountConnections>,
}

#[derive(Debug, Clone)]
pub struct TestContext {
    config: Arc<Config>,
    test_config: Arc<TestMode>,
    state: Arc<Mutex<State>>,
}

impl TestContext {
    pub fn new(config: Arc<Config>, test_config: Arc<TestMode>) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                connections: vec![],
            })),
            config,
            test_config,
        }
    }

    pub async fn close_websocket_connections(&mut self) {
        let mut state = self.state.lock().await;
        let mut connections = Vec::new();
        mem::swap(&mut connections, &mut state.connections);
        for connections in connections.into_iter() {
            connections.close().await;
        }
    }

    /// Account with InitialSetup state.
    pub async fn new_account_in_initial_setup_state(&self) -> Result<Account, TestError> {
        Account::register_and_login(self.clone()).await
    }

    /// Account with Normal state.
    pub async fn new_account(&self) -> Result<Account, TestError> {
        let mut account = Account::register_and_login(self.clone()).await?;
        account
            .run_actions(action_array![
                SetAccountSetup::new(),
                SendImageToSlot::slot(0),
                SendImageToSlot::slot(1),
                SetPendingContent {
                    security_content_slot_i: Some(0),
                    content_0_slot_i: Some(1),
                },
                MakeModerationRequest {
                    slot_0_secure_capture: true
                },
                CompleteAccountSetup,
            ])
            .await?;
        Ok(account)
    }

    pub async fn new_account_with_settings(
        &self,
        age: i64,
        name: &str,
        min_age: i32,
        max_age: i32,
        groups: SearchGroups,
    ) -> Result<Account, TestError> {
        let account = self.new_account().await?;
        let update = ProfileUpdate {
            attributes: vec![],
            age,
            name: name.to_string(),
            profile_text: String::new(),
        };
        post_profile(account.profile_api(), update)
            .await
            .change_context(TestError::ApiRequest)?;

        let range = ProfileSearchAgeRange {
            min: min_age,
            max: max_age,
        };
        post_search_age_range(account.profile_api(), range)
            .await
            .change_context(TestError::ApiRequest)?;

        post_search_groups(account.profile_api(), groups)
            .await
            .change_context(TestError::ApiRequest)?;

        Ok(account)
    }

    pub async fn new_man_18_years(&self) -> Result<Account, TestError> {
        self.new_account_with_settings(
            18,
            "M",
            18,
            18,
            SearchGroups {
                man_for_woman: Some(true),
                ..SearchGroups::default()
            },
        )
        .await
    }

    pub async fn new_man_4_man_18_years(&self) -> Result<Account, TestError> {
        self.new_account_with_settings(
            18,
            "M",
            18,
            18,
            SearchGroups {
                man_for_man: Some(true),
                ..SearchGroups::default()
            },
        )
        .await
    }

    pub async fn new_woman_18_years(&self) -> Result<Account, TestError> {
        self.new_account_with_settings(
            18,
            "W",
            18,
            18,
            SearchGroups {
                woman_for_man: Some(true),
                ..SearchGroups::default()
            },
        )
        .await
    }

    /// Admin account with Normal state.
    pub async fn new_admin(&self) -> Result<Admin, TestError> {
        let mut account = Account::register_and_login(self.clone()).await?;
        account
            .run_actions(action_array![
                SetAccountSetup::admin(),
                SendImageToSlot::slot(0),
                SendImageToSlot::slot(1),
                SetPendingContent {
                    security_content_slot_i: Some(0),
                    content_0_slot_i: Some(1),
                },
                MakeModerationRequest {
                    slot_0_secure_capture: true
                },
                CompleteAccountSetup,
            ])
            .await?;
        Ok(Admin { account })
    }

    /// Admin account with Normal state.
    pub async fn new_admin_and_moderate_initial_content(&self) -> Result<Admin, TestError> {
        let mut admin = self.new_admin().await?;
        admin.accept_initial_content_moderation_requests().await?;
        Ok(admin)
    }

    async fn add_account_connections(&mut self, connections: AccountConnections) {
        let mut state = self.state.lock().await;
        state.connections.push(connections);
    }
}

pub struct Account {
    test_context: TestContext,
    bot_state: BotState,
}

impl Account {
    pub async fn register_and_login(mut test_context: TestContext) -> Result<Self, TestError> {
        let mut state = BotState::new(
            None,
            test_context.config.clone(),
            test_context.test_config.clone(),
            Arc::new(BotConfigFile::default()),
            0,
            0,
            ApiClient::new(test_context.test_config.server.api_urls.clone()),
        );
        state.connections.enable_event_sending = true;

        Register.excecute_impl(&mut state).await?;
        Login.excecute_impl(&mut state).await?;

        let connections = state.connections.unwrap_account_connections();
        test_context.add_account_connections(connections).await;

        Ok(Self {
            test_context,
            bot_state: state,
        })
    }

    pub fn register_api(&self) -> &Configuration {
        self.bot_state.api.register()
    }

    pub fn account_api(&self) -> &Configuration {
        self.bot_state.api.account()
    }

    pub fn profile_api(&self) -> &Configuration {
        self.bot_state.api.profile()
    }

    pub fn media_api(&self) -> &Configuration {
        self.bot_state.api.media()
    }

    pub fn chat_api(&self) -> &Configuration {
        self.bot_state.api.chat()
    }

    pub fn account_id(&self) -> AccountId {
        self.bot_state.id.unwrap()
    }

    pub fn account_id_string(&self) -> String {
        self.account_id().account_id.to_string()
    }

    /// Only actions without TaskState usage are supported
    pub async fn run<T: BotAction>(&mut self, action: T) -> Result<(), TestError> {
        action.excecute_impl(&mut self.bot_state).await
    }

    /// Only actions without TaskState usage are supported
    pub async fn run_actions(&mut self, actions: &[&dyn BotAction]) -> Result<(), TestError> {
        for action in actions.iter() {
            action.excecute_impl(&mut self.bot_state).await?;
        }
        Ok(())
    }

    /// Debug print BotState partially
    pub fn print(&self) {
        println!("BotState media {:#?}", self.bot_state.media);
    }

    /// Wait event if event sending enabled or timeout after 5 seconds
    pub async fn wait_event(
        &mut self,
        check: impl Fn(&EventToClient) -> bool,
    ) -> Result<(), TestError> {
        self.bot_state.wait_event(check).await
    }
}

pub struct Admin {
    account: Account,
}

impl Admin {
    pub fn account(&self) -> &Account {
        &self.account
    }

    pub fn account_mut(&mut self) -> &mut Account {
        &mut self.account
    }

    pub async fn accept_initial_content_moderation_requests(&mut self) -> Result<(), TestError> {
        self.accept_content_moderation_requests(ModerationQueueType::InitialMediaModeration)
            .await
    }

    pub async fn accept_content_moderation_requests(
        &mut self,
        queue: ModerationQueueType,
    ) -> Result<(), TestError> {
        loop {
            self.account
                .run(ModerateMediaModerationRequest::from_queue(queue))
                .await?;

            let list =
                media_admin_api::patch_moderation_request_list(self.account.media_api(), queue)
                    .await
                    .change_context(TestError::ApiRequest)?;

            if list.list.is_empty() {
                break;
            }
        }
        Ok(())
    }
}
