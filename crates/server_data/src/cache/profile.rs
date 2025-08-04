use std::sync::Arc;

use error_stack::{Result, ResultExt};
use model::{
    AccountId, AccountIdInternal, AutomaticProfileSearchCompletedNotification, NextNumberStorage,
    UnixTime,
};
use model_server_data::{
    AtomicLastSeenTime, AutomaticProfileSearchIteratorSessionIdInternal,
    AutomaticProfileSearchLastSeenUnixTime, LastSeenUnixTime, ProfileAppNotificationSettings,
    ProfileAttributeFilterValue, ProfileAttributeValue, ProfileCreatedTimeFilter,
    ProfileEditedTimeFilter, ProfileInternal, ProfileIteratorSessionIdInternal,
    ProfileNameModerationState, ProfileQueryMakerDetails, ProfileStateCached,
    ProfileTextCharacterCount, ProfileTextModerationState, ProfileVersion, SortedProfileAttributes,
};
use server_common::data::{DataError, cache::CacheError};

use crate::{
    db_manager::InternalWriting,
    index::{coordinates::LocationIndexArea, read::LocationIndexIteratorState},
};

#[derive(Debug)]
pub struct CacheProfile {
    pub account_id: AccountId,
    data: ProfileInternal,
    pub state: ProfileStateCached,
    pub location: LocationData,
    pub attributes: SortedProfileAttributes,
    pub attribute_filters: Vec<ProfileAttributeFilterValue>,
    last_seen_time: Arc<AtomicLastSeenTime>,
    pub profile_iterator_session_id: Option<ProfileIteratorSessionIdInternal>,
    pub profile_iterator_session_id_storage: NextNumberStorage,
    pub automatic_profile_search: AutomaticProifleSearch,
    profile_name_moderation_state: Option<ProfileNameModerationState>,
    profile_text_character_count: ProfileTextCharacterCount,
    profile_text_moderation_state: Option<ProfileTextModerationState>,
}

impl CacheProfile {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account_id: AccountId,
        data: ProfileInternal,
        state: ProfileStateCached,
        attributes: Vec<ProfileAttributeValue>,
        attribute_filters: Vec<ProfileAttributeFilterValue>,
        last_seen_time: LastSeenUnixTime,
        automatic_profile_search_last_seen_time: Option<AutomaticProfileSearchLastSeenUnixTime>,
        profile_name_moderation_state: Option<ProfileNameModerationState>,
        profile_text_moderation_state: Option<ProfileTextModerationState>,
    ) -> Self {
        Self {
            account_id,
            profile_text_character_count: ProfileTextCharacterCount::new(&data),
            data,
            state,
            location: LocationData::default(),
            attributes: SortedProfileAttributes::new(attributes),
            attribute_filters,
            last_seen_time: AtomicLastSeenTime::new(last_seen_time).into(),
            profile_iterator_session_id: None,
            profile_iterator_session_id_storage: NextNumberStorage::default(),
            automatic_profile_search: AutomaticProifleSearch::new(
                automatic_profile_search_last_seen_time,
            ),
            profile_name_moderation_state,
            profile_text_moderation_state,
        }
    }

    pub fn profile_internal(&self) -> &ProfileInternal {
        &self.data
    }

    pub fn profile_text_character_count(&self) -> ProfileTextCharacterCount {
        self.profile_text_character_count
    }

    pub fn update_profile_version_uuid(&mut self, v: ProfileVersion) {
        self.data.version_uuid = v;
    }

    pub fn update_profile_name(&mut self, v: String) {
        self.data.profile_name = v;
    }

    pub fn update_profile_internal(&mut self, action: impl FnOnce(&mut ProfileInternal)) {
        action(&mut self.data);
        self.profile_text_character_count = ProfileTextCharacterCount::new(&self.data);
    }

    pub fn filters(&self) -> ProfileQueryMakerDetails {
        ProfileQueryMakerDetails::new(&self.data, &self.state, self.attribute_filters.clone())
    }

    pub fn automatic_profile_search_filters(
        &self,
        settings: &ProfileAppNotificationSettings,
    ) -> ProfileQueryMakerDetails {
        ProfileQueryMakerDetails::new_for_automatic_profile_search(
            &self.data,
            &self.state,
            &self.attribute_filters,
            settings,
            || self.automatic_profile_search.profile_created_time_filter(),
            || self.automatic_profile_search.profile_edited_time_filter(),
        )
    }

    pub fn last_seen_time(&self) -> &Arc<AtomicLastSeenTime> {
        &self.last_seen_time
    }

    pub fn profile_name_moderation_state(&self) -> Option<ProfileNameModerationState> {
        self.profile_name_moderation_state
    }

    pub fn update_profile_name_moderation_state(
        &mut self,
        value: Option<ProfileNameModerationState>,
    ) {
        self.profile_name_moderation_state = value;
    }

    pub fn profile_text_moderation_state(&self) -> Option<ProfileTextModerationState> {
        self.profile_text_moderation_state
    }

    pub fn update_profile_text_moderation_state(
        &mut self,
        value: Option<ProfileTextModerationState>,
    ) {
        self.profile_text_moderation_state = value;
    }
}

#[derive(Debug, Clone)]
pub struct LocationData {
    pub current_position: LocationIndexArea,
    pub current_iterator: LocationIndexIteratorState,
}

impl Default for LocationData {
    fn default() -> Self {
        Self {
            current_position: LocationIndexArea::default(),
            current_iterator: LocationIndexIteratorState::completed(),
        }
    }
}

pub trait UpdateLocationCacheState {
    async fn update_location_cache_profile(&self, id: AccountIdInternal) -> Result<(), DataError>;
}

impl<I: InternalWriting> UpdateLocationCacheState for I {
    async fn update_location_cache_profile(&self, id: AccountIdInternal) -> Result<(), DataError> {
        let (location, profile_data, profile_visibility) = self
            .cache()
            .read_cache(id.as_id(), |e| {
                let profile_visibility = e
                    .common
                    .account_state_related_shared_state
                    .profile_visibility();
                let p = e.profile.as_deref().ok_or(CacheError::FeatureNotEnabled)?;
                Ok((
                    p.location.current_position.profile_location(),
                    e.location_index_profile_data()?,
                    profile_visibility,
                ))
            })
            .await
            .change_context(DataError::Cache)?;

        if profile_visibility.is_currently_public() {
            self.location_index_write_handle()
                .update_profile_data(id.as_id(), profile_data, location)
                .await
                .change_context(DataError::ProfileIndex)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct AutomaticProifleSearch {
    pub current_iterator: LocationIndexIteratorState,
    pub iterator_session_id: Option<AutomaticProfileSearchIteratorSessionIdInternal>,
    pub iterator_session_id_storage: NextNumberStorage,
    last_seen_unix_time: Option<AutomaticProfileSearchLastSeenUnixTime>,
    pub notification: AutomaticProfileSearchCompletedNotification,
}

impl AutomaticProifleSearch {
    fn new(last_seen_unix_time: Option<AutomaticProfileSearchLastSeenUnixTime>) -> Self {
        Self {
            current_iterator: LocationIndexIteratorState::completed(),
            iterator_session_id: None,
            iterator_session_id_storage: NextNumberStorage::default(),
            last_seen_unix_time,
            notification: AutomaticProfileSearchCompletedNotification::default(),
        }
    }

    fn profile_edited_time_filter(&self) -> Option<ProfileEditedTimeFilter> {
        self.last_seen_unix_time.map(|v| {
            let current_time = UnixTime::current_time();
            let seconds_since_last_seen = *current_time.as_i64() - *v.as_i64();
            ProfileEditedTimeFilter {
                value: seconds_since_last_seen,
            }
        })
    }

    fn profile_created_time_filter(&self) -> Option<ProfileCreatedTimeFilter> {
        self.last_seen_unix_time.map(|v| {
            let current_time = UnixTime::current_time();
            let seconds_since_last_seen = *current_time.as_i64() - *v.as_i64();
            ProfileCreatedTimeFilter {
                value: seconds_since_last_seen,
            }
        })
    }

    pub fn last_seen_unix_time(&self) -> Option<AutomaticProfileSearchLastSeenUnixTime> {
        self.last_seen_unix_time
    }

    pub fn update_last_seen_unix_time(&mut self, time: AutomaticProfileSearchLastSeenUnixTime) {
        self.last_seen_unix_time = Some(time);
    }
}
