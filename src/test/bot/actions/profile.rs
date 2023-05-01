use std::{fmt::Debug, collections::HashSet};

use api_client::{apis::profile_api::{post_profile, self}, models::{ProfileUpdate, Location}};
use async_trait::async_trait;
use error_stack::Result;

use super::{super::super::client::TestError, BotAction, PreviousValue};

use crate::{utils::IntoReportExt, config::file::LocationConfig};

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

#[derive(Debug)]
pub struct UpdateLocation(pub Location);

#[async_trait]
impl BotAction for UpdateLocation {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        profile_api::put_location(state.api.profile(), self.0)
            .await
            .into_error(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateLocationRandom(pub LocationConfig);

#[async_trait]
impl BotAction for UpdateLocationRandom {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        profile_api::put_location(state.api.profile(), self.0.generate_random_location())
            .await
            .into_error(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ResetProfileIterator;

#[async_trait]
impl BotAction for ResetProfileIterator {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        profile_api::post_reset_profile_paging(state.api.profile())
            .await
            .into_error(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct GetProfileList;

#[async_trait]
impl BotAction for GetProfileList {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let data = profile_api::post_get_next_profile_page(state.api.profile())
            .await
            .into_error(TestError::ApiRequest)?;
        let value = HashSet::<String>::from_iter(data.profiles.into_iter().map(|l| l.id.to_string()));
        state.previous_value = PreviousValue::Profiles(value);
        Ok(())
    }

    fn previous_value_supported(&self) -> bool { true }
}
