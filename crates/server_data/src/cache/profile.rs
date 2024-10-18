use config::Config;
use model::{AccountId, ProfileIteratorSessionIdInternal, LocationIndexKey, NextNumberStorage, ProfileAttributeFilterValue, ProfileAttributeValue, ProfileInternal, ProfileQueryMakerDetails, ProfileStateCached, ProfileStateInternal, SortedProfileAttributes, UnixTime};

use crate::index::location::LocationIndexIteratorState;

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
}

impl CachedProfile {
    pub fn new(
        account_id: AccountId,
        data: ProfileInternal,
        state: ProfileStateInternal,
        attributes: Vec<ProfileAttributeValue>,
        filters: Vec<ProfileAttributeFilterValue>,
        config: &Config,
        last_seen_time: Option<UnixTime>,
    ) -> Self {
        Self {
            account_id,
            data,
            state: state.into(),
            location: LocationData {
                current_position: LocationIndexKey::default(),
                current_iterator: LocationIndexIteratorState::new(),
            },
            attributes: SortedProfileAttributes::new(attributes, config.profile_attributes()),
            filters,
            last_seen_time,
            profile_iterator_session_id: None,
            profile_iterator_session_id_storage: NextNumberStorage::default(),
        }
    }

    pub fn filters(&self) -> ProfileQueryMakerDetails {
        ProfileQueryMakerDetails::new(&self.data, &self.state, self.filters.clone())
    }
}

#[derive(Debug, Clone)]
pub struct LocationData {
    pub current_position: LocationIndexKey,
    pub current_iterator: LocationIndexIteratorState,
}
