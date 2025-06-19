use std::{collections::HashSet, fmt::Debug};

use api_client::{
    apis::profile_api::{self, get_location, get_profile, post_profile},
    models::{Location, ProfileAttributeValueUpdate, ProfileIteratorSessionId, ProfileUpdate},
};
use async_trait::async_trait;
use chrono::{DateTime, NaiveTime, Utc};
use config::file::LocationConfig;
use error_stack::{Result, ResultExt};
use tracing::error;

use super::{super::super::client::TestError, BotAction, BotState, PreviousValue};
use crate::bot::utils::location::LocationConfigUtils;

#[derive(Debug, Default)]
pub struct ProfileState {
    profile_iterator_session_id: Option<ProfileIteratorSessionId>,
    change_profile_text_daily: Option<DateTime<Utc>>,
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
                // Uuid has same string size every time.
                simple_backend_utils::UuidBase64Url::new_random_id().to_string()
            }
        };
        let update = ProfileUpdate {
            attributes: current_profile
                .attributes
                .unwrap_or_default()
                .iter()
                .map(|a| ProfileAttributeValueUpdate {
                    id: a.id,
                    v: a.v.clone(),
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
pub struct ChangeProfileTextDaily;

#[async_trait]
impl BotAction for ChangeProfileTextDaily {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let Some(config) = state.get_bot_config().change_profile_text_time() else {
            return Ok(());
        };

        let Some(time) = NaiveTime::from_hms_opt(config.0.hours.into(), config.0.minutes.into(), 0)
        else {
            error!("NaiveTime creation failed");
            return Ok(());
        };

        let current_time = Utc::now();
        let Some(next) = current_time.with_time(time).single() else {
            error!("Next profile text update time creation failed");
            return Ok(());
        };

        let update = if let Some(previous) = state.profile.change_profile_text_daily {
            previous.date_naive() != next.date_naive() && current_time > next
        } else {
            current_time > next
        };

        if update {
            state.profile.change_profile_text_daily = Some(current_time);
            let config = state.get_bot_config();
            let time_text = current_time.to_rfc2822();
            let new_text = if let Some(text) = &config.text {
                format!("{}\n{}", text, time_text)
            } else {
                time_text
            };
            ChangeProfileText {
                mode: ProfileText::String(new_text),
            }
            .excecute_impl(state)
            .await?;
        }

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

/// Updates location with random values if bot is not configured to specific
/// location.
/// If None is passed, then area for random location is
/// from Config.
///
/// Updates PreviousValue to a new location.
#[derive(Debug)]
pub struct UpdateLocationRandomOrConfigured {
    pub config: Option<LocationConfig>,
    pub deterministic: bool,
}

impl UpdateLocationRandomOrConfigured {
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
impl BotAction for UpdateLocationRandomOrConfigured {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let config = self
            .config
            .clone()
            .unwrap_or(state.server_config.location().clone());
        let mut location = config.generate_random_location(if self.deterministic {
            Some(&mut state.deterministic_rng)
        } else {
            None
        });
        if let Some(lat) = state.get_bot_config().lat {
            location.latitude = lat;
        }
        if let Some(lon) = state.get_bot_config().lon {
            location.longitude = lon;
        }
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
        let data =
            profile_api::post_get_next_profile_page(state.api.profile(), iterator_session_id)
                .await
                .change_context(TestError::ApiRequest)?;
        let value =
            HashSet::<String>::from_iter(data.profiles.into_iter().map(|l| l.a.to_string()));
        state.previous_value = PreviousValue::Profiles(value);
        Ok(())
    }

    fn previous_value_supported(&self) -> bool {
        true
    }
}
