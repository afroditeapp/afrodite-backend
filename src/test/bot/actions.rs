use std::fmt::{Debug, Display};

use api_client::{apis::{account_api::{post_register, post_login}, profile_api::{post_profile, get_profile, get_default_profile}}, models::Profile};
use async_trait::async_trait;
use nalgebra::U8;



use error_stack::{Result, FutureExt, ResultExt};

use tracing::{error, log::warn};

use super::super::client::{ApiClient, TestError};

use crate::{
    api::model::AccountId,
    config::args::{Test, TestMode},
    utils::IntoReportExt,
};

use super::BotState;


#[async_trait]
pub trait BotAction: Debug + Send + Sync + 'static {
    async fn excecute(&self, state: &mut BotState) -> Result<(), TestError> {
        self
            .excecute_impl(state)
            .await
            .attach_printable_lazy(|| format!("{:?}", self))
    }

    async fn excecute_impl(
        &self,
        state: &mut BotState
    ) -> Result<(), TestError>;
}






#[derive(Debug)]
pub struct DoNothing;

#[async_trait]
impl BotAction for DoNothing {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        Ok(())
    }
}

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
pub struct ChangeProfileText;

#[async_trait]
impl BotAction for ChangeProfileText {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let profile = rand::random::<u32>();
        let profile = Profile::new(format!("{}", profile));
        post_profile(state.api.profile(), profile)
            .await
            .into_error(TestError::ApiRequest)?;
        Ok(())
    }
}
