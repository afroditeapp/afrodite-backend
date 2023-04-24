pub mod account;
pub mod profile;
pub mod media;
pub mod admin;

use std::{fmt::{Debug, Display}, time::Duration};

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
pub struct AssertFailure<T: BotAction>(pub T);

#[async_trait]
impl <T: BotAction> BotAction for AssertFailure<T> {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        match self.0.excecute(state).await {
            Err(e) => match e.current_context() {
                TestError::ApiRequest => Ok(()),
                _ => Err(e),
            },
            Ok(()) => Err(TestError::AssertError("API request did not fail".to_string()).into()),
        }
    }
}


/// Sleep milliseconds
#[derive(Debug)]
pub struct SleepMillis(pub u64);

#[async_trait]
impl BotAction for SleepMillis {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        tokio::time::sleep(Duration::from_millis(self.0)).await;
        Ok(())
    }
}
