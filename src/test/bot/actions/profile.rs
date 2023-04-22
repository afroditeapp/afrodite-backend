
use std::fmt::{Debug, Display};

use api_client::{apis::{account_api::{post_register, post_login}, profile_api::{post_profile, get_profile, get_default_profile}}, models::Profile};
use async_trait::async_trait;
use nalgebra::U8;

use error_stack::{Result, FutureExt, ResultExt};

use tracing::{error, log::warn};

use super::{super::super::client::{ApiClient, TestError}, BotAction};

use crate::{
    api::model::AccountId,
    config::args::{Test, TestMode},
    utils::IntoReportExt,
};

use super::BotState;


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
