use std::{collections::HashSet, fmt::Debug};

use api_client::{
    apis::{
        common_api::get_client_config,
        profile_api::{
            self, get_location, get_profile, post_get_query_profile_attributes_config,
            post_profile, post_search_age_range, post_search_groups,
        },
    },
    models::{
        AttributeMode, Location, ProfileAttributeValueUpdate, ProfileAttributesConfigQuery,
        ProfileIteratorSessionId, ProfileUpdate, SearchAgeRange, SearchGroups,
    },
};
use async_trait::async_trait;
use chrono::{DateTime, NaiveTime, Utc};
use config::{bot_config_file::Gender, file::LocationConfig};
use error_stack::{Result, ResultExt};
use test_mode_utils::{
    client::TestError,
    websocket_protocol::{ResponseNextProfilePage, ResponseResetProfilePaging},
};
use tracing::error;
use utils::minimal_i64;

use super::{BotAction, BotState, PreviousValue};
use crate::{actions::account::DEFAULT_AGE, utils::location::LocationConfigUtils};

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
        let current_profile = get_profile(&state.api(), &id, None, None)
            .await
            .change_context(TestError::ApiRequest)?
            .profile
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
            name: current_profile.name.unwrap_or_default(),
            ptext: Some(profile_text),
        };
        post_profile(&state.api(), update)
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
                format!("{text}\n{time_text}")
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
        let profile = get_profile(&state.api(), &state.account_id_string()?, None, None)
            .await
            .change_context(TestError::ApiRequest)?
            .profile
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
        profile_api::put_location(&state.api(), self.0)
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
        let location = get_location(&state.api())
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
///
/// If [Self::config] is None, then area for random location is
/// [config::bot_config_file::BotConfigFile::location] or
/// default [LocationConfig].
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
            .unwrap_or(state.bot_config_file.location.clone().unwrap_or_default());
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
        profile_api::put_location(&state.api(), location)
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
        let request_id = state.next_ws_request_id();
        let response = wait_response_reset_profile_paging(
            state,
            create_request_reset_profile_paging_message(request_id),
            request_id,
        )
        .await?;
        if !response.success {
            return Err(TestError::ApiRequest.report());
        }

        let iterator_session_id = response
            .iterator_session_id
            .ok_or(TestError::MissingValue)?;
        state.profile.profile_iterator_session_id =
            Some(ProfileIteratorSessionId::new(iterator_session_id));
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

        let request_id = state.next_ws_request_id();
        let response = wait_response_next_profile_page(
            state,
            create_request_get_next_profile_page_message(request_id, &iterator_session_id),
            request_id,
        )
        .await?;
        if !response.success {
            return Err(TestError::ApiRequest.report());
        }

        let value = HashSet::<String>::from_iter(response.profiles);
        state.previous_value = PreviousValue::Profiles(value);
        Ok(())
    }

    fn previous_value_supported(&self) -> bool {
        true
    }
}

fn create_request_get_next_profile_page_message(
    request_id: u8,
    iterator_session_id: &ProfileIteratorSessionId,
) -> Vec<u8> {
    const REQUEST_GET_NEXT_PROFILE_PAGE: u8 = 61;

    let mut payload = vec![REQUEST_GET_NEXT_PROFILE_PAGE, request_id];
    minimal_i64::add_minimal_i64(&mut payload, iterator_session_id.id);
    payload
}

fn create_request_reset_profile_paging_message(request_id: u8) -> Vec<u8> {
    const REQUEST_RESET_PROFILE_PAGING: u8 = 60;
    vec![REQUEST_RESET_PROFILE_PAGING, request_id]
}

async fn wait_response_reset_profile_paging(
    state: &mut BotState,
    message: Vec<u8>,
    request_id: u8,
) -> Result<ResponseResetProfilePaging, TestError> {
    let mut message = Some(message);

    loop {
        tokio::select! {
            _ = tokio::task::yield_now(), if message.is_some() => {
                if let Some(message) = message.take() {
                    state.connections.send_client_message(message)?;
                }
            }
            event = state.connections.recv_event_unchecked() => {
                let event = event?;
                if let Some(response) = event.response_reset_profile_paging && response.request_id == request_id {
                    return Ok(response);
                }
            }
        }
    }
}

async fn wait_response_next_profile_page(
    state: &mut BotState,
    message: Vec<u8>,
    request_id: u8,
) -> Result<ResponseNextProfilePage, TestError> {
    let mut message = Some(message);

    loop {
        tokio::select! {
            _ = tokio::task::yield_now(), if message.is_some() => {
                if let Some(message) = message.take() {
                    state.connections.send_client_message(message)?;
                }
            }
            event = state.connections.recv_event_unchecked() => {
                let event = event?;
                if let Some(response) = event.response_next_profile_page && response.request_id == request_id {
                    return Ok(response);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ChangeBotAgeAndOtherSettings {
    pub admin: bool,
}

#[async_trait]
impl BotAction for ChangeBotAgeAndOtherSettings {
    async fn excecute_impl(&self, state: &mut BotState) -> Result<(), TestError> {
        let bot_config = state.get_bot_config();
        let age = bot_config.age.unwrap_or(DEFAULT_AGE);

        let groups = {
            let man = SearchGroups {
                man_for_man: Some(true),
                man_for_woman: Some(true),
                man_for_non_binary: Some(true),
                ..Default::default()
            };
            let woman = SearchGroups {
                woman_for_man: Some(true),
                woman_for_woman: Some(true),
                woman_for_non_binary: Some(true),
                ..Default::default()
            };
            let non_binary = SearchGroups {
                non_binary_for_man: Some(true),
                non_binary_for_woman: Some(true),
                non_binary_for_non_binary: Some(true),
                ..Default::default()
            };

            match bot_config.gender {
                Some(Gender::Man) => man,
                Some(Gender::Woman) => woman,
                None => match state.task_id % 3 {
                    0 => man,
                    1 => woman,
                    _ => non_binary,
                },
            }
        };

        let available_attributes = get_client_config(&state.api())
            .await
            .change_context(TestError::ApiRequest)?
            .profile_attributes
            .flatten()
            .map(|v| v.attributes)
            .unwrap_or_default();

        let available_attributes = post_get_query_profile_attributes_config(
            &state.api(),
            ProfileAttributesConfigQuery {
                values: available_attributes.iter().map(|v| v.id).collect(),
            },
        )
        .await
        .change_context(TestError::ApiRequest)?
        .values
        .into_iter()
        .map(|v| v.a);

        let mut attributes: Vec<ProfileAttributeValueUpdate> = vec![];
        for attribute in available_attributes {
            if attribute.required.unwrap_or_default() && attribute.mode == AttributeMode::Bitflag {
                let max_selected = attribute
                    .max_selected
                    .and_then(|v| usize::try_from(v).ok())
                    .unwrap_or(1);

                let mut selected = 0;
                for value in attribute.values.into_iter().take(max_selected) {
                    selected |= value.id;
                }

                let update = ProfileAttributeValueUpdate {
                    id: attribute.id,
                    v: vec![selected],
                };

                attributes.push(update);
            }
        }

        let name = if self.admin {
            format!("Admin bot {}", state.task_id + 1)
        } else {
            state
                .get_bot_config()
                .name
                .clone()
                .map(|v| v.into_string())
                .unwrap_or("B".to_string())
        };

        let update = ProfileUpdate {
            name,
            age: age.into(),
            attributes,
            ptext: state.get_bot_config().text.clone().map(|v| v.into_string()),
        };

        post_profile(&state.api(), update)
            .await
            .change_context(TestError::ApiRequest)?;

        let age_range = SearchAgeRange { min: 18, max: 99 };

        post_search_age_range(&state.api(), age_range)
            .await
            .change_context(TestError::ApiRequest)?;

        post_search_groups(&state.api(), groups)
            .await
            .change_context(TestError::ApiRequest)?;

        Ok(())
    }
}
