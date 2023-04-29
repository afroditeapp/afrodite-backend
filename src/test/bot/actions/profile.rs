
use std::fmt::{Debug};

use api_client::{apis::{profile_api::{post_profile}}, models::{ProfileUpdate}};
use async_trait::async_trait;
use error_stack::{Result};



use super::{super::super::client::{TestError}, BotAction};

use crate::{
    utils::IntoReportExt,
};

use super::BotState;


#[derive(Debug)]
pub struct ChangeProfileText;

#[async_trait]
impl BotAction for ChangeProfileText {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let profile = rand::random::<u32>();
        let profile = ProfileUpdate::new(format!("{}", profile));
        post_profile(state.api.profile(), profile)
            .await
            .into_error(TestError::ApiRequest)?;
        Ok(())
    }
}
