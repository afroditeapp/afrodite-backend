use std::{collections::HashSet, fmt::Debug};

use api_client::{
    apis::profile_api::{self, get_location, get_profile, post_profile},
    models::{IteratorSessionId, Location, ProfileAttributeValueUpdate, ProfileUpdate},
};
use async_trait::async_trait;
use config::file::LocationConfig;
use error_stack::{Result, ResultExt};

use super::{super::super::client::TestError, BotAction, BotState, PreviousValue};
use crate::bot::utils::location::LocationConfigUtils;

#[derive(Debug, Default)]
pub struct ProfileState {
    profile_iterator_session_id: Option<IteratorSessionId>,
}

impl ProfileState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub enum ProfileText {
    Static(&'static str),
    String(String),
    Random,
}

#[derive(Debug)]
pub struct ChangeProfileText {
    pub mode: ProfileText,
}

#[async_trait]
impl BotAction for ChangeProfileText {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let id = state.account_id_string()?;
        let current_profile = get_profile(state.api.profile(), &id, None, None)
            .await
            .change_context(TestError::ApiRequest)?
            .p
            .flatten()
            .ok_or(TestError::MissingValue.report())?
            .as_ref()
            .clone();

        let profile_text = match &self.mode {
            ProfileText::Static(text) => text.to_string(),
            ProfileText::String(text) => text.clone(),
            ProfileText::Random => {
                uuid::Uuid::new_v4().to_string() // Uuid has same string size every time.
            }
        };
        let update = ProfileUpdate {
            attributes: current_profile
                .attributes
                .unwrap_or_default()
                .iter()
                .map(|a| ProfileAttributeValueUpdate {
                    id: a.id,
                    values: a.values.clone(),
                })
                .collect(),
            age: current_profile.age,
            name: current_profile.name,
            ptext: profile_text,
        };
        post_profile(state.api.profile(), update)
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
        let profile = get_profile(state.api.profile(), &state.account_id_string()?, None, None)
            .await
            .change_context(TestError::ApiRequest)?
            .p
            .flatten()
            .ok_or(TestError::MissingValue.report())?
            .as_ref()
            .clone();
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
pub struct GetLocation;

#[async_trait]
impl BotAction for GetLocation {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let location = get_location(state.api.profile())
            .await
            .change_context(TestError::ApiRequest)?;
        state.previous_value = PreviousValue::Location(location);
        Ok(())
    }

    fn previous_value_supported(&self) -> bool {
        true
    }
}

/// Updates location with random values.
/// If None is passed, then area for random location is
/// from Config.
///
/// Updates PreviousValue to a new location.
#[derive(Debug)]
pub struct UpdateLocationRandom {
    pub config: Option<LocationConfig>,
    pub deterministic: bool,
}

impl UpdateLocationRandom {
    pub const fn new(config: Option<LocationConfig>) -> Self {
        Self {
            config,
            deterministic: false,
        }
    }

    pub const fn new_deterministic(config: Option<LocationConfig>) -> Self {
        Self {
            config,
            deterministic: true,
        }
    }
}

#[async_trait]
impl BotAction for UpdateLocationRandom {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let config = self
            .config
            .clone()
            .unwrap_or(state.server_config.location().clone());
        let location = config.generate_random_location(
            if self.deterministic {
                Some(&mut state.deterministic_rng)
            } else {
                None
            }
        );
        profile_api::put_location(state.api.profile(), location)
            .await
            .change_context(TestError::ApiRequest)?;
        state.previous_value = PreviousValue::Location(location);
        Ok(())
    }

    fn previous_value_supported(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct ResetProfileIterator;

#[async_trait]
impl BotAction for ResetProfileIterator {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let iterator_session_id = profile_api::post_reset_profile_paging(state.api.profile())
            .await
            .change_context(TestError::ApiRequest)?;
        state.profile.profile_iterator_session_id = Some(iterator_session_id);
        Ok(())
    }
}

#[derive(Debug)]
pub struct GetProfileList;

#[async_trait]
impl BotAction for GetProfileList {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let iterator_session_id = state
            .profile
            .profile_iterator_session_id
            .as_ref()
            .ok_or(TestError::MissingValue)?
            .clone();
        let data = profile_api::post_get_next_profile_page(state.api.profile(), iterator_session_id)
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
