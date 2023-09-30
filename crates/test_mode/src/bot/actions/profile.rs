use std::{collections::HashSet, fmt::Debug};

use api_client::{
    apis::profile_api::{self, post_profile, get_profile},
    models::{Location, ProfileUpdate},
};
use async_trait::async_trait;
use config::file::LocationConfig;
use error_stack::{Result, ResultExt};


use super::{super::super::client::TestError, BotAction, BotState, PreviousValue};
use crate::bot::utils::location::LocationConfigUtils;

#[derive(Debug)]
pub enum ProfileText {
    Static(&'static str),
    Random,
}

#[derive(Debug)]
pub struct ChangeProfileText {
    pub mode: ProfileText,
}

#[async_trait]
impl BotAction for ChangeProfileText {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let profile = match self.mode {
            ProfileText::Static(text) => text.to_string(),
            ProfileText::Random => {
                uuid::Uuid::new_v4().to_string() // Uuid has same string size every time.
            }
        };
        let profile = ProfileUpdate::new(format!("{}", profile));
        post_profile(state.api.profile(), profile)
            .await
            .change_context(TestError::ApiRequest)?;
        Ok(())
    }
}


#[derive(Debug)]
pub struct GetProfile;

#[async_trait]
impl BotAction for GetProfile {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let profile = get_profile(state.api.profile(), &state.account_id_string()?)
            .await
            .change_context(TestError::ApiRequest)?;
        state.previous_value = PreviousValue::Profile(profile);
        Ok(())
    }

    fn previous_value_supported(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct UpdateLocation(pub Location);

#[async_trait]
impl BotAction for UpdateLocation {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        profile_api::put_location(state.api.profile(), self.0)
            .await
            .change_context(TestError::ApiRequest)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct GetLocation();

#[async_trait]
impl BotAction for GetLocation {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        // TODO: Update bindings and use get_location
        Ok(())
    }

    fn previous_value_supported(&self) -> bool {
        true
    }
}

/// Updates location with random values.
/// If None is passed, then area for random location is
/// from Config.
#[derive(Debug)]
pub struct UpdateLocationRandom(pub Option<LocationConfig>);

#[async_trait]
impl BotAction for UpdateLocationRandom {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let config = self.0
            .clone()
            .unwrap_or(state.server_config.location().clone());

        profile_api::put_location(state.api.profile(), config.generate_random_location())
            .await
            .change_context(TestError::ApiRequest)?;
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
            .change_context(TestError::ApiRequest)?;
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
            .change_context(TestError::ApiRequest)?;
        let value =
            HashSet::<String>::from_iter(data.profiles.into_iter().map(|l| l.id.to_string()));
        state.previous_value = PreviousValue::Profiles(value);
        Ok(())
    }

    fn previous_value_supported(&self) -> bool {
        true
    }
}
