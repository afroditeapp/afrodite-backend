use std::{num::NonZeroU16, sync::atomic::{AtomicI64, AtomicU64, Ordering}};

use model::{AccountId, InitialSetupCompletedTime, ProfileAge, ProfileContentVersion};
use nalgebra::DMatrix;
use simple_backend_model::UnixTime;

use super::{
    LastSeenTimeFilter, ProfileAttributeFilterValue, ProfileAttributesInternal, ProfileCreatedTimeFilter, ProfileEditedTime, ProfileEditedTimeFilter, ProfileInternal, ProfileSearchAgeRangeValidated, ProfileStateCached, ProfileTextCharacterCount, ProfileTextMaxCharactersFilter, ProfileTextMinCharactersFilter, SearchGroupFlags, SearchGroupFlagsFilter, SortedProfileAttributes
};
use crate::{LastSeenTime, ProfileAppNotificationSettings, ProfileContentEditedTime, ProfileLink};

#[derive(Debug)]
pub struct ProfileQueryMakerDetails {
    pub age: ProfileAge,
    pub search_age_range: ProfileSearchAgeRangeValidated,
    pub search_groups_filter: SearchGroupFlagsFilter,
    pub attribute_filters: Vec<ProfileAttributeFilterValue>,
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
    pub unlimited_likes_filter: Option<bool>,
    pub profile_created_time_filter: Option<ProfileCreatedTimeFilter>,
    pub profile_edited_time_filter: Option<ProfileEditedTimeFilter>,
    pub profile_text_min_characters_filter: Option<ProfileTextMinCharactersFilter>,
    pub profile_text_max_characters_filter: Option<ProfileTextMaxCharactersFilter>,
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
            profile_created_time_filter: state.profile_created_time_filter,
            profile_edited_time_filter: state.profile_edited_time_filter,
            profile_text_min_characters_filter: state.profile_text_min_characters_filter,
            profile_text_max_characters_filter: state.profile_text_max_characters_filter,
        }
    }

    pub fn new_for_automatic_profile_search(
        profile: &ProfileInternal,
        state: &ProfileStateCached,
        attribute_filters: &[ProfileAttributeFilterValue],
        settings: &ProfileAppNotificationSettings,
        profile_created_time_filter: impl FnOnce() -> Option<ProfileCreatedTimeFilter>,
        profile_edited_time_filter: impl FnOnce() -> Option<ProfileEditedTimeFilter>,
    ) -> Self {
        Self {
            age: profile.age,
            search_age_range: ProfileSearchAgeRangeValidated::new(
                state.search_age_range_min,
                state.search_age_range_max,
            ),
            search_groups_filter: state.search_group_flags.to_filter(),
            attribute_filters: if settings.automatic_profile_search_filters {
                attribute_filters.to_vec()
            } else {
                vec![]
            },
            last_seen_time_filter: None,
            unlimited_likes_filter: None,
            profile_created_time_filter: if settings.automatic_profile_search_new_profiles {
                profile_created_time_filter()
            } else {
                None
            },
            profile_edited_time_filter: if settings.automatic_profile_search_new_profiles {
                None
            } else {
                profile_edited_time_filter()
            },
            profile_text_min_characters_filter: None,
            profile_text_max_characters_filter: None,
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
    profile_created_time: InitialSetupCompletedTime,
    profile_edited_time: ProfileEditedTime,
    /// Option because media component might not be enabled
    profile_content_edited_time: Option<ProfileContentEditedTime>,
    profile_text_character_count: ProfileTextCharacterCount,
}

impl LocationIndexProfileData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: AccountId,
        profile: &ProfileInternal,
        state: &ProfileStateCached,
        attributes: SortedProfileAttributes,
        profile_content_version: Option<ProfileContentVersion>,
        unlimited_likes: bool,
        last_seen_value: Option<LastSeenTime>,
        profile_created_time: InitialSetupCompletedTime,
        profile_content_edited_time: Option<ProfileContentEditedTime>,
        profile_text_character_count: ProfileTextCharacterCount,
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
            profile_created_time,
            profile_edited_time: state.profile_edited_time,
            profile_content_edited_time,
            profile_text_character_count,
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
        attribute_info: Option<&ProfileAttributesInternal>,
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
            if let Some(profile_created_time_filter) = query_maker_details.profile_created_time_filter {
                is_match &= profile_created_time_filter.is_match(self.profile_created_time, current_time);
            }
        }

        if is_match {
            if let Some(profile_edited_time_filter) = query_maker_details.profile_edited_time_filter {
                is_match &= profile_edited_time_filter.is_match(self.profile_edited_time, self.profile_content_edited_time, current_time);
            }
        }

        if is_match {
            if let Some(filter) = query_maker_details.profile_text_min_characters_filter {
                is_match &= filter.is_match(self.profile_text_character_count);
            }
        }

        if is_match {
            if let Some(filter) = query_maker_details.profile_text_max_characters_filter {
                is_match &= filter.is_match(self.profile_text_character_count);
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
        attribute_info: &ProfileAttributesInternal,
    ) -> bool {
        for filter in &query_maker_details.attribute_filters {
            let attribute_info =
                if let Some(info) = attribute_info.get_attribute(filter.id()) {
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

impl std::ops::Index<LocationIndexKey> for DMatrix<CellData> {
    type Output = <Self as std::ops::Index<(usize, usize)>>::Output;

    fn index(&self, key: LocationIndexKey) -> &Self::Output {
        &self[(key.y as usize, key.x as usize)]
    }
}

struct BitFieldInfo {
    mask: u64,
    shift: u64,
}

#[derive(Debug)]
pub struct CellData {
    /// Contains these values starting from least significant bit:
    ///
    /// - next_up (u15)
    /// - profiles_in_this_area (bit flag)
    /// - next_down (u15)
    /// - empty (bit flag)
    /// - next_left (u15)
    /// - empty (bit flag)
    /// - next_right (u15)
    /// - empty (bit flag)
    pub state: AtomicU64,
}

impl CellData {
    const NEXT_UP: BitFieldInfo = BitFieldInfo {
        mask: 0x7FFF,
        shift: 0,
    };
    const NEXT_DOWN: BitFieldInfo = BitFieldInfo {
        mask: 0x7FFF_0000,
        shift: 16,
    };
    const NEXT_LEFT: BitFieldInfo = BitFieldInfo {
        mask: 0x7FFF_0000_0000,
        shift: 32,
    };
    const NEXT_RIGHT: BitFieldInfo = BitFieldInfo {
        mask: 0x7FFF_0000_0000_0000,
        shift: 48,
    };

    const PROFILES_IN_THIS_AREA_MASK: u64 = 0x8000;

    pub fn new(width: NonZeroU16, height: NonZeroU16) -> Self {
        let mut state: u64 = 0;
        state |= ((height.get() - 1) as u64) << Self::NEXT_DOWN.shift;
        state |= ((width.get() - 1) as u64) << Self::NEXT_RIGHT.shift;
        Self {
            state: AtomicU64::new(state),
        }
    }

    fn state(&self) -> u64 {
        self.state.load(Ordering::Relaxed)
    }

    fn update_bit_field(&self, i: usize, info: BitFieldInfo) {
        let mut state = self.state() & !info.mask;
        state |= ((i as u64) & 0x7FFF) << info.shift;
        self.state.store(state, Ordering::Relaxed)
    }

    pub fn set_next_up(&self, i: usize) {
        self.update_bit_field(i, Self::NEXT_UP);
    }

    pub fn set_next_down(&self, i: usize) {
        self.update_bit_field(i, Self::NEXT_DOWN);
    }

    pub fn set_next_left(&self, i: usize) {
        self.update_bit_field(i, Self::NEXT_LEFT);
    }

    pub fn set_next_right(&self, i: usize) {
        self.update_bit_field(i, Self::NEXT_RIGHT);
    }

    pub fn set_profiles(&self, value: bool) {
        if value {
            self.state.fetch_or(Self::PROFILES_IN_THIS_AREA_MASK, Ordering::Relaxed);
        } else {
            self.state.fetch_and(!Self::PROFILES_IN_THIS_AREA_MASK, Ordering::Relaxed);
        }
    }

    fn parser(&self) -> CellDataParser {
        CellDataParser(self.state())
    }
}

pub struct CellDataParser(u64);

impl CellDataParser {
    fn read_bit_field(&self, field: BitFieldInfo) -> u64 {
        (self.0 & field.mask) >> field.shift
    }
    fn next_up(&self) -> u64 {
        self.read_bit_field(CellData::NEXT_UP)
    }
    fn next_down(&self) -> u64 {
        self.read_bit_field(CellData::NEXT_DOWN)
    }
    fn next_left(&self) -> u64 {
        self.read_bit_field(CellData::NEXT_LEFT)
    }
    fn next_right(&self) -> u64 {
        self.read_bit_field(CellData::NEXT_RIGHT)
    }
    fn profiles(&self) -> bool {
        (self.0 & CellData::PROFILES_IN_THIS_AREA_MASK) != 0
    }
}

pub struct CellState {
    pub next_up: isize,
    pub next_down: isize,
    pub next_left: isize,
    pub next_right: isize,
    pub profiles_in_this_area: bool,
}

pub trait CellDataProvider {
    fn next_up(&self) -> usize;
    fn next_down(&self) -> usize;
    fn next_left(&self) -> usize;
    fn next_right(&self) -> usize;
    fn profiles(&self) -> bool;
    fn state(&self) -> CellState;
}

impl CellDataProvider for CellData {
    fn next_up(&self) -> usize {
        self.parser().next_up() as usize
    }

    fn next_down(&self) -> usize {
        self.parser().next_down() as usize
    }

    fn next_left(&self) -> usize {
        self.parser().next_left() as usize
    }

    fn next_right(&self) -> usize {
        self.parser().next_right() as usize
    }

    fn profiles(&self) -> bool {
        self.parser().profiles()
    }

    fn state(&self) -> CellState {
        let parser = self.parser();
        CellState {
            next_up: parser.next_up() as isize,
            next_down: parser.next_down() as isize,
            next_left: parser.next_left() as isize,
            next_right: parser.next_right() as isize,
            profiles_in_this_area: parser.profiles(),
        }
    }
}
