use config::Config;
use error_stack::{Result, ResultExt};
use model::{AccountId, AccountIdInternal, AutomaticProfileSearchCompletedNotification, NextNumberStorage, UnixTime};
use model_server_data::{
    LastSeenTime, ProfileAttributeFilterValue, ProfileAttributeValue, ProfileEditedTimeFilter, ProfileInternal, ProfileIteratorSessionIdInternal, ProfileQueryMakerDetails, ProfileStateCached, SortedProfileAttributes
};
use server_common::data::{cache::CacheError, DataError};

use crate::{
    cache::CacheEntryCommon, db_manager::InternalWriting,
    index::{area::LocationIndexArea, location::LocationIndexIteratorState},
};

#[derive(Debug)]
pub struct CachedProfile {
    pub account_id: AccountId,
    pub data: ProfileInternal,
    pub state: ProfileStateCached,
    pub location: LocationData,
    pub attributes: SortedProfileAttributes,
    pub filters: Vec<ProfileAttributeFilterValue>,
    pub last_seen_time: Option<UnixTime>,
    pub profile_iterator_session_id: Option<ProfileIteratorSessionIdInternal>,
    pub profile_iterator_session_id_storage: NextNumberStorage,
    pub automatic_profile_search: AutomaticProifleSearch,
}

impl CachedProfile {
    pub fn new(
        account_id: AccountId,
        data: ProfileInternal,
        state: ProfileStateCached,
        attributes: Vec<ProfileAttributeValue>,
        filters: Vec<ProfileAttributeFilterValue>,
        config: &Config,
        last_seen_time: Option<UnixTime>,
    ) -> Self {
        Self {
            account_id,
            data,
            state,
            location: LocationData::default(),
            attributes: SortedProfileAttributes::new(attributes, config.profile_attributes()),
            filters,
            last_seen_time,
            profile_iterator_session_id: None,
            profile_iterator_session_id_storage: NextNumberStorage::default(),
            automatic_profile_search: AutomaticProifleSearch::default(),
        }
    }

    pub fn filters(&self) -> ProfileQueryMakerDetails {
        ProfileQueryMakerDetails::new(&self.data, &self.state, self.filters.clone())
    }

    pub fn automatic_profile_search_filters(&self) -> ProfileQueryMakerDetails {
        let mut filters = ProfileQueryMakerDetails::new(&self.data, &self.state, self.filters.clone());
        filters.attribute_filters = vec![];
        filters.last_seen_time_filter = None;
        filters.unlimited_likes_filter = None;
        filters.profile_created_time_filter = None;
        filters.profile_edited_time_filter = self.automatic_profile_search.profile_edited_time_filter();
        filters
    }

    pub fn last_seen_time_for_db(&self) -> Option<UnixTime> {
        self.last_seen_time
    }

    pub fn last_seen_time(&self, common: &CacheEntryCommon) -> Option<LastSeenTime> {
        if common.current_connection.is_some() {
            Some(LastSeenTime::ONLINE)
        } else {
            self.last_seen_time.map(|v| v.into())
        }
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
    pub profile_iterator_session_id: Option<ProfileIteratorSessionIdInternal>,
    pub profile_iterator_session_id_storage: NextNumberStorage,
    pub last_seen_unix_time: Option<UnixTime>,
    pub notification: AutomaticProfileSearchCompletedNotification,
}

impl Default for AutomaticProifleSearch {
    fn default() -> Self {
        Self {
            current_iterator: LocationIndexIteratorState::completed(),
            profile_iterator_session_id: None,
            profile_iterator_session_id_storage: NextNumberStorage::default(),
            last_seen_unix_time: None,
            notification: AutomaticProfileSearchCompletedNotification::default(),
        }
    }
}

impl AutomaticProifleSearch {
    fn profile_edited_time_filter(&self) -> Option<ProfileEditedTimeFilter> {
        self.last_seen_unix_time.map(|v| {
            let current_time = UnixTime::current_time();
            let seconds_since_last_seen = *current_time.as_i64() - *v.as_i64();
            ProfileEditedTimeFilter { value: seconds_since_last_seen }
        })
    }
}
