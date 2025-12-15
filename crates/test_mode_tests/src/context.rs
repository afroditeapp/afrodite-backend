use std::{mem, sync::Arc};

use api_client::{
    apis::{
        configuration::Configuration,
        profile_api::{post_profile, post_search_age_range, post_search_groups},
    },
    models::{
        AccountId, EventToClient, ModerationQueueType, ProfileUpdate, SearchAgeRange, SearchGroups,
    },
};
use config::{args::TestMode, bot_config_file::BotConfigFile};
use error_stack::{Result, ResultExt};
use test_mode_bot::{
    BotState, action_array,
    actions::{
        BotAction,
        account::{CompleteAccountSetup, Login, Register, SetAccountSetup},
        admin::content::ModerateContentModerationRequest,
        media::{SendImageToSlot, SetContent},
    },
    connection::ApiConnection,
};
use test_mode_utils::client::{ApiClient, TestError};
use tokio::sync::Mutex;

#[derive(Debug)]
struct State {
    pub connections: Vec<ApiConnection>,
}

#[derive(Debug, Clone)]
pub struct TestContext {
    test_config: Arc<TestMode>,
    state: Arc<Mutex<State>>,
    account_server_api_port: Option<u16>,
    next_bot_id: u32,
    admin_access_granted: bool,
    reqwest_client: reqwest::Client,
}

impl TestContext {
    pub fn new(
        test_config: Arc<TestMode>,
        account_server_api_port: Option<u16>,
        reqwest_client: reqwest::Client,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                connections: vec![],
            })),
            test_config,
            account_server_api_port,
            next_bot_id: 1, // 0 is for admin bot
            admin_access_granted: false,
            reqwest_client,
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
    pub async fn new_account_in_initial_setup_state(&mut self) -> Result<Account, TestError> {
        Account::register_and_login(self, false).await
    }

    /// Account with Normal state, age 30 and name "Test".
    pub async fn new_account(&mut self) -> Result<Account, TestError> {
        self.new_account_internal(30, "Test").await
    }

    async fn new_account_internal(&mut self, age: i64, name: &str) -> Result<Account, TestError> {
        let mut account = Account::register_and_login(self, false).await?;
        account
            .run_actions(action_array![
                SetAccountSetup::new(),
                SendImageToSlot::slot(0),
                SetContent {
                    security_content_slot_i: Some(0),
                    content_0_slot_i: Some(0),
                },
            ])
            .await?;

        let update = ProfileUpdate {
            attributes: vec![],
            age,
            name: name.to_string(),
            ptext: None,
        };
        post_profile(account.profile_api(), update)
            .await
            .change_context(TestError::ApiRequest)?;

        account
            .run_actions(action_array![CompleteAccountSetup,])
            .await?;

        Ok(account)
    }

    pub async fn new_account_with_settings(
        &mut self,
        age: i64,
        name: &str,
        min_age: i32,
        max_age: i32,
        groups: SearchGroups,
    ) -> Result<Account, TestError> {
        let account = self.new_account_internal(age, name).await?;

        let range = SearchAgeRange {
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

    pub async fn new_man_18_years(&mut self) -> Result<Account, TestError> {
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

    pub async fn new_man_4_man_18_years(&mut self) -> Result<Account, TestError> {
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

    pub async fn new_woman_18_years(&mut self) -> Result<Account, TestError> {
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
    pub async fn new_admin(&mut self) -> Result<Admin, TestError> {
        let mut account = Account::register_and_login(self, true).await?;
        account
            .run_actions(action_array![
                SetAccountSetup::admin(),
                SendImageToSlot::slot(0),
                SetContent {
                    security_content_slot_i: Some(0),
                    content_0_slot_i: Some(0),
                },
                CompleteAccountSetup,
            ])
            .await?;
        Ok(Admin { account })
    }

    /// Admin account with Normal state.
    pub async fn new_admin_and_moderate_initial_content(&mut self) -> Result<Admin, TestError> {
        let mut admin = self.new_admin().await?;
        admin
            .accept_pending_content_moderations_for_initial_images()
            .await?;
        Ok(admin)
    }

    async fn add_account_connections(&mut self, connections: ApiConnection) {
        let mut state = self.state.lock().await;
        state.connections.push(connections);
    }
}

pub struct Account {
    bot_state: BotState,
}

impl Account {
    pub async fn register_and_login(
        test_context: &mut TestContext,
        admin: bool,
    ) -> Result<Self, TestError> {
        let urls = test_context
            .test_config
            .api_urls
            .clone()
            .change_ports(test_context.account_server_api_port)
            .map_err(|_| TestError::ApiUrlPortConfigFailed.report())?;

        let bot_id = if admin && !test_context.admin_access_granted {
            let id = 0;
            test_context.admin_access_granted = true;
            id
        } else {
            let id = test_context.next_bot_id;
            test_context.next_bot_id += 1;
            id
        };

        let mut state = BotState::new(
            None,
            None,
            test_context.test_config.clone(),
            Arc::new(BotConfigFile::default()),
            0,
            bot_id,
            ApiClient::new(urls.clone(), &test_context.reqwest_client),
            urls,
            test_context.reqwest_client.clone(),
        );
        state.enable_events();

        Register.excecute_impl(&mut state).await?;
        Login.excecute_impl(&mut state).await?;

        let connections = state.connections.unwrap_account_connections();
        test_context.add_account_connections(connections).await;

        Ok(Self { bot_state: state })
    }

    pub fn account_api(&self) -> &Configuration {
        self.bot_state.api()
    }

    pub fn profile_api(&self) -> &Configuration {
        self.bot_state.api()
    }

    pub fn media_api(&self) -> &Configuration {
        self.bot_state.api()
    }

    pub fn chat_api(&self) -> &Configuration {
        self.bot_state.api()
    }

    pub fn account_id(&self) -> AccountId {
        self.bot_state.id.clone().unwrap()
    }

    pub fn account_id_string(&self) -> String {
        self.account_id().aid.to_string()
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

    pub async fn accept_pending_content_moderations_for_initial_images(
        &mut self,
    ) -> Result<(), TestError> {
        self.accept_pending_content_moderations(ModerationQueueType::InitialMediaModeration)
            .await
    }

    pub async fn accept_pending_content_moderations(
        &mut self,
        queue: ModerationQueueType,
    ) -> Result<(), TestError> {
        self.account
            .run(ModerateContentModerationRequest::from_queue(queue, true))
            .await?;
        Ok(())
    }
}
