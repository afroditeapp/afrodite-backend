use std::fmt::Debug;

use api_client::{
    apis::account_api::{
        get_account_state, post_account_setup, post_complete_setup, post_login, post_register,
    },
    models::{AccountSetup, AccountState},
};
use async_trait::async_trait;

use error_stack::Result;

use super::{super::super::client::TestError, BotAction};

use crate::{
    test::bot::utils::{assert::bot_assert_eq, name::NameProvider},
    utils::IntoReportExt,
};

use super::BotState;

#[derive(Debug)]
pub struct Register;

#[async_trait]
impl BotAction for Register {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        if state.id.is_some() {
            return Ok(());
        }

        let id = post_register(state.api.account())
            .await
            .into_error(TestError::ApiRequest)?;
        state.id = Some(id);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Login;

#[async_trait]
impl BotAction for Login {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        if state.api.is_api_key_available() {
            return Ok(());
        }
        let key = post_login(state.api.account(), state.id()?)
            .await
            .into_error(TestError::ApiRequest)?;

        state.api.set_api_key(key);
        Ok(())
    }
}

#[derive(Debug)]
pub struct AssertAccountState(pub AccountState);

#[async_trait]
impl BotAction for AssertAccountState {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let state = get_account_state(state.api.account())
            .await
            .into_error(TestError::ApiRequest)?;

        bot_assert_eq(state.state, self.0)
    }
}

#[derive(Debug)]
pub struct SetAccountSetup {
    pub email: Option<&'static str>,
}

impl SetAccountSetup {
    pub const fn new() -> Self {
        Self { email: None }
    }

    pub const fn admin() -> &'static dyn BotAction {
        &Self {
            email: Some("admin@example.com"),
        }
    }
}

#[async_trait]
impl BotAction for SetAccountSetup {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let name = NameProvider::men_first_name().to_string();
        let setup = AccountSetup {
            email: self
                .email
                .map(|email| email.to_string())
                .unwrap_or(format!("{}@example.com", &name)),
            name,
        };
        post_account_setup(state.api.account(), setup)
            .await
            .into_error(TestError::ApiRequest)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct CompleteAccountSetup;

#[async_trait]
impl BotAction for CompleteAccountSetup {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        post_complete_setup(state.api.account())
            .await
            .into_error(TestError::ApiRequest)?;

        Ok(())
    }
}
