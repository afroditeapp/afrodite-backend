use std::sync::Arc;

use api_client::{apis::configuration::Configuration, models::AccountId};
use config::{args::TestMode, Config};
use error_stack::Result;
use tokio::sync::Mutex;

use crate::{
    bot::{
        actions::{
            account::{Login, Register},
            BotAction,
        },
        BotState, WsConnection,
    },
    client::ApiClient,
    TestError,
};

#[derive(Debug)]
struct State {
    pub web_sockets: Vec<WsConnection>,
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
                web_sockets: vec![],
            })),
            config,
            test_config,
        }
    }

    pub async fn clear(&mut self) {
        let mut state = self.state.lock().await;

        for ws in state.web_sockets.iter_mut() {
            let _ = ws.close(None).await;
        }

        state.web_sockets.clear();
    }

    pub async fn new_account(&self) -> Result<Account, TestError> {
        Account::register_and_login(self.clone()).await
    }

    async fn add_web_socket(&mut self, ws: WsConnection) {
        let mut state = self.state.lock().await;
        state.web_sockets.push(ws);
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
            0,
            0,
            ApiClient::new(test_context.test_config.server.api_urls.clone()),
        );

        Register.excecute_impl(&mut state).await?;
        Login.excecute_impl(&mut state).await?;

        let connections = state.connections.take_connections();
        for ws in connections {
            test_context.add_web_socket(ws).await;
        }

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
}
