use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU16, Ordering};

use model::{AccountId, ProfileAge, ProfileContentVersion};
use nalgebra::DMatrix;
use simple_backend_model::UnixTime;

use super::{
    LastSeenTimeFilter, ProfileAttributeFilterValue, ProfileAttributes, ProfileInternal,
    ProfileSearchAgeRangeValidated, ProfileStateCached, SearchGroupFlags, SearchGroupFlagsFilter,
    SortedProfileAttributes,
};
use crate::{LastSeenTime, ProfileLink};

#[derive(Debug)]
pub struct ProfileQueryMakerDetails {
    pub age: ProfileAge,
    pub search_age_range: ProfileSearchAgeRangeValidated,
    pub search_groups_filter: SearchGroupFlagsFilter,
    pub attribute_filters: Vec<ProfileAttributeFilterValue>,
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
    pub unlimited_likes_filter: Option<bool>,
}

impl ProfileQueryMakerDetails {
    pub fn new(
        profile: &ProfileInternal,
        state: &ProfileStateCached,
        attribute_filters: Vec<ProfileAttributeFilterValue>,
    ) -> Self {
        Self {
            age: profile.age,
            search_age_range: ProfileSearchAgeRangeValidated::new(
                state.search_age_range_min,
                state.search_age_range_max,
            ),
            search_groups_filter: state.search_group_flags.to_filter(),
            attribute_filters,
            last_seen_time_filter: state.last_seen_time_filter,
            unlimited_likes_filter: state.unlimited_likes_filter,
        }
    }
}

/// All data which location index needs for returning filtered profiles when
/// user queries new profiles.
#[derive(Debug)]
pub struct LocationIndexProfileData {
    /// Simple profile information for client.
    profile_link: ProfileLink,
    age: ProfileAge,
    search_age_range: ProfileSearchAgeRangeValidated,
    search_groups: SearchGroupFlags,
    attributes: SortedProfileAttributes,
    unlimited_likes: bool,
    /// Possible values:
    /// - Unix timestamp
    /// - Value -1 is currently online
    /// - i64::MIN is None
    last_seen_time: AtomicI64,
}

impl LocationIndexProfileData {
    pub fn new(
        id: AccountId,
        profile: &ProfileInternal,
        state: &ProfileStateCached,
        attributes: SortedProfileAttributes,
        profile_content_version: Option<ProfileContentVersion>,
        unlimited_likes: bool,
        last_seen_value: Option<LastSeenTime>,
    ) -> Self {
        Self {
            profile_link: ProfileLink::new(id, profile.version_uuid, profile_content_version, None),
            age: profile.age,
            search_age_range: ProfileSearchAgeRangeValidated::new(
                state.search_age_range_min,
                state.search_age_range_max,
            ),
            search_groups: state.search_group_flags,
            attributes,
            unlimited_likes,
            last_seen_time: if let Some(last_seen_time) = last_seen_value {
                AtomicI64::new(last_seen_time.raw())
            } else {
                AtomicI64::new(i64::MIN)
            },
        }
    }

    pub fn to_profile_link_value(&self) -> ProfileLink {
        let mut profile_link = self.profile_link;
        let last_seen_value = self.last_seen_time.load(Ordering::Relaxed);
        if last_seen_value >= LastSeenTime::MIN_VALUE {
            profile_link.set_last_seen_time(LastSeenTime::new(last_seen_value));
        }
        profile_link
    }

    pub fn update_last_seen_value(&self, value: LastSeenTime) {
        self.last_seen_time.store(value.raw(), Ordering::Relaxed);
    }

    pub fn is_match(
        &self,
        query_maker_details: &ProfileQueryMakerDetails,
        attribute_info: Option<&ProfileAttributes>,
        current_time: &UnixTime,
    ) -> bool {
        let mut is_match = self.search_age_range.is_match(query_maker_details.age)
            && query_maker_details.search_age_range.is_match(self.age)
            && query_maker_details
                .search_groups_filter
                .is_match(self.search_groups);

        if is_match {
            if let Some(last_seen_time_filter) = query_maker_details.last_seen_time_filter {
                is_match &= self.last_seen_time_match(last_seen_time_filter, current_time);
            }
        }

        if is_match {
            if let Some(unlimited_likes_filter) = query_maker_details.unlimited_likes_filter {
                is_match &= unlimited_likes_filter == self.unlimited_likes;
            }
        }

        if is_match {
            if let Some(attribute_info) = attribute_info {
                is_match &= self.attribute_filters_match(query_maker_details, attribute_info);
            }
        }

        is_match
    }

    fn last_seen_time_match(
        &self,
        last_seen_time_filter: LastSeenTimeFilter,
        current_time: &UnixTime,
    ) -> bool {
        let current_last_seen_time = self.last_seen_time.load(Ordering::Relaxed);
        let current_last_seen_time = if current_last_seen_time < -1 {
            return false;
        } else {
            LastSeenTime::new(current_last_seen_time)
        };

        last_seen_time_filter.is_match(current_last_seen_time, current_time)
    }

    fn attribute_filters_match(
        &self,
        query_maker_details: &ProfileQueryMakerDetails,
        attribute_info: &ProfileAttributes,
    ) -> bool {
        for filter in &query_maker_details.attribute_filters {
            let attribute_info =
                if let Some(info) = attribute_info.attributes.get(filter.id() as usize) {
                    info
                } else {
                    return false;
                };

            if let Some(value) = self.attributes.find_id(filter.id()) {
                if !filter.is_match_with_attribute_value(value, attribute_info) {
                    return false;
                }
            } else {
                if !filter.accept_missing_attribute_enabled() {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Debug, Hash, PartialEq, Clone, Copy, Default, Eq)]
pub struct LocationIndexKey {
    pub y: u16,
    pub x: u16,
}

impl LocationIndexKey {
    pub fn x(&self) -> usize {
        self.x as usize
    }

    pub fn y(&self) -> usize {
        self.y as usize
    }
}

#[derive(Debug)]
pub struct CellData {
    pub next_up: AtomicU16,
    pub next_down: AtomicU16,
    pub next_left: AtomicU16,
    pub next_right: AtomicU16,
    pub profiles_in_this_area: AtomicBool,
}

impl std::ops::Index<LocationIndexKey> for DMatrix<CellData> {
    type Output = <Self as std::ops::Index<(usize, usize)>>::Output;

    fn index(&self, key: LocationIndexKey) -> &Self::Output {
        &self[(key.y as usize, key.x as usize)]
    }
}

impl CellData {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            next_down: AtomicU16::new(height.checked_sub(1).unwrap()),
            next_up: AtomicU16::new(0),
            next_left: AtomicU16::new(0),
            next_right: AtomicU16::new(width.checked_sub(1).unwrap()),
            profiles_in_this_area: AtomicBool::new(false),
        }
    }

    pub fn set_next_down(&self, i: usize) {
        self.next_down.store(i as u16, Ordering::Relaxed)
    }

    pub fn set_next_up(&self, i: usize) {
        self.next_up.store(i as u16, Ordering::Relaxed)
    }

    pub fn set_next_left(&self, i: usize) {
        self.next_left.store(i as u16, Ordering::Relaxed)
    }

    pub fn set_next_right(&self, i: usize) {
        self.next_right.store(i as u16, Ordering::Relaxed)
    }

    pub fn set_profiles(&self, value: bool) {
        self.profiles_in_this_area.store(value, Ordering::Relaxed)
    }
}

pub trait CellDataProvider {
    fn next_down(&self) -> usize;
    fn next_up(&self) -> usize;
    fn next_left(&self) -> usize;
    fn next_right(&self) -> usize;
    fn profiles(&self) -> bool;
}

impl CellDataProvider for CellData {
    fn next_down(&self) -> usize {
        self.next_down.load(Ordering::Relaxed) as usize
    }

    fn next_up(&self) -> usize {
        self.next_up.load(Ordering::Relaxed) as usize
    }

    fn next_left(&self) -> usize {
        self.next_left.load(Ordering::Relaxed) as usize
    }

    fn next_right(&self) -> usize {
        self.next_right.load(Ordering::Relaxed) as usize
    }

    fn profiles(&self) -> bool {
        self.profiles_in_this_area.load(Ordering::Relaxed)
    }
}
