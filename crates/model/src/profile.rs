use std::{
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicBool, AtomicU16, Ordering},
};

use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary},
    AsExpression, FromSqlRow,
};
use nalgebra::DMatrix;
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_struct_try_from, diesel_i64_wrapper, diesel_uuid_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{
    schema_sqlite_types::Integer, sync_version_wrappers, AccountId, AccountIdDb, SyncVersion, SyncVersionUtils, ProfileContentVersion,
};

mod attribute;

pub use attribute::*;

const NUMBER_LIST_ATTRIBUTE_MAX_VALUES: usize = 8;

/// Profile's database data
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::profile)]
#[diesel(check_for_backend(crate::Db))]
pub struct ProfileInternal {
    pub account_id: AccountIdDb,
    pub version_uuid: ProfileVersion,
    pub name: String,
    pub profile_text: String,
    pub age: ProfileAge,
}

impl ProfileInternal {
    pub fn update_from(&mut self, update: &ProfileUpdateValidated) {
        self.name.clone_from(&update.name);
        self.profile_text.clone_from(&update.profile_text);
        self.age = update.age;
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeValueUpdate {
    /// Attribute ID
    pub id: u16,
    /// Empty list removes the attribute.
    ///
    /// - First value is bitflags value or top level attribute value ID or first number list value.
    /// - Second value is sub level attribute value ID or second number list value.
    /// - Third and rest are number list values.
    pub values: Vec<u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeValue {
    /// Attribute ID
    id: u16,
    /// - First value is bitflags value or top level attribute value ID or first number list value.
    /// - Second value is sub level attribute value ID or second number list value.
    /// - Third and rest are number list values.
    ///
    /// The number list values are in ascending order.
    values: Vec<u16>,
}

impl ProfileAttributeValue {
    pub fn try_from_update_and_sort(mut value: ProfileAttributeValueUpdate, attribute: &Attribute) -> Result<Self, String> {
        if attribute.mode.is_number_list() {
            value.values.sort();
        }
        Self::try_from_update(value)
    }

    pub fn try_from_update(value: ProfileAttributeValueUpdate) -> Result<Self, String> {
        match value.values.first() {
            Some(_) => Ok(Self { id: value.id, values: value.values }),
            None => Err("Value part1 missing".to_string()),
        }
    }

    pub fn new_not_number_list(id: u16, values: Vec<u16>) -> Self {
        Self { id, values }
    }

    pub fn new_number_list(id: u16, mut values: Vec<u16>) -> Self {
        values.sort();
        Self { id, values }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn as_bitflags(&self) -> u16 {
        self.values.first().copied().unwrap_or(0)
    }

    /// ID number for top level AttributeValue ID.
    pub fn as_top_level_id(&self) -> u16 {
        self.values.first().copied().unwrap_or(0)
    }

    /// ID number for sub level AttributeValue ID.
    pub fn as_sub_level_id(&self) -> Option<u16> {
        self.values.get(1).copied()
    }

    pub fn as_number_list(&self) -> &[u16] {
        &self.values
    }
}

impl From<ProfileAttributeValue> for ProfileAttributeValueUpdate {
    fn from(value: ProfileAttributeValue) -> Self {
        Self {
            id: value.id,
            values: value.values,
        }
    }
}

/// The profile attributes and possible number list values are sorted.
#[derive(Debug, Clone, PartialEq)]
pub struct SortedProfileAttributes {
    attributes: Vec<ProfileAttributeValue>,
}

impl SortedProfileAttributes {
    pub fn new(attributes: Vec<ProfileAttributeValue>, all_attributes: Option<&ProfileAttributes>) -> Self {
        let mut attributes = attributes;
        attributes.sort_by(|a, b| a.id.cmp(&b.id));

        for a in &mut attributes {
            let id_usize: usize = a.id.into();
            if let Some(info) = all_attributes.and_then(|attributes| attributes.attributes.get(id_usize)) {
                if info.mode.is_number_list() {
                    a.values.sort();
                }
            }
        }

        Self { attributes }
    }

    pub fn attributes(&self) -> &Vec<ProfileAttributeValue> {
        &self.attributes
    }

    pub fn find_id(&self, id: u16) -> Option<&ProfileAttributeValue> {
        self.attributes
            .binary_search_by(|a| a.id.cmp(&id))
            .ok()
            .and_then(|i| self.attributes.get(i))
    }

    pub fn update_from(&mut self, update: &ProfileUpdateValidated) {
        let mut attributes = update
            .attributes
            .iter()
            .filter_map(|v| ProfileAttributeValue::try_from_update(v.clone()).ok())
            .collect::<Vec<_>>();
        attributes.sort_by(|a, b| a.id.cmp(&b.id));
        self.attributes = attributes;
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeFilterListUpdate {
    filters: Vec<ProfileAttributeFilterValueUpdate>,
}

impl ProfileAttributeFilterListUpdate {
    pub fn validate(
        self,
        attribute_info: Option<&ProfileAttributes>,
    ) -> Result<ProfileAttributeFilterListUpdateValidated, String> {
        let mut hash_set = HashSet::new();
        for a in &self.filters {
            if !hash_set.insert(a.id) {
                return Err("Duplicate attribute ID".to_string());
            }

            if let Some(info) = attribute_info {
                let attribute_info = info.attributes.get(a.id as usize);
                match attribute_info {
                    None => return Err("Unknown attribute ID".to_string()),
                    Some(info) => {
                        if info.mode.is_number_list() && a.filter_values.len() > NUMBER_LIST_ATTRIBUTE_MAX_VALUES {
                            return Err(format!("Number list attribute supports max {} filters", NUMBER_LIST_ATTRIBUTE_MAX_VALUES));
                        }
                    }
                }
            } else {
                return Err("Profile attributes are disabled".to_string());
            }
        }

        Ok(ProfileAttributeFilterListUpdateValidated {
            filters: self.filters,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeFilterListUpdateValidated {
    pub filters: Vec<ProfileAttributeFilterValueUpdate>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeFilterValueUpdate {
    /// Attribute ID
    pub id: u16,
    /// - First value is bitflags value or top level attribute value ID or first number list value.
    /// - Second value is sub level attribute value ID or second number list value.
    /// - Third and rest are number list values.
    pub filter_values: Vec<u16>,
    /// Defines should missing attribute be accepted.
    ///
    /// Setting this to `None` disables the filter.
    pub accept_missing_attribute: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeFilterList {
    pub filters: Vec<ProfileAttributeFilterValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileAttributeFilterValue {
    /// Attribute ID
    id: u16,
    /// - First value is bitflags value or top level attribute value ID or first number list value.
    /// - Second value is sub level attribute value ID or second number list value.
    /// - Third and rest are number list values.
    ///
    /// The number list values are in ascending order.
    filter_values: Vec<u16>,
    accept_missing_attribute: bool,
}

impl ProfileAttributeFilterValue {
    pub fn new_not_number_list(
        id: u16,
        filter_values: Vec<u16>,
        accept_missing_attribute: bool,
    ) -> Self {
        Self {
            id,
            filter_values,
            accept_missing_attribute,
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn accept_missing_attribute_enabled(&self) -> bool {
        self.accept_missing_attribute
    }

    /// Bitflag filter value
    pub fn as_bitflags(&self) -> u16 {
        self.filter_values.first().copied().unwrap_or(0)
    }

    /// ID number for top level AttributeValue ID.
    pub fn as_top_level_id(&self) -> Option<u16> {
        self.filter_values.first().copied()
    }

    /// ID number for sub level AttributeValue ID.
    pub fn as_sub_level_id(&self) -> Option<u16> {
        self.filter_values.get(1).copied()
    }

    pub fn as_number_list(&self) -> &[u16] {
        &self.filter_values
    }

    pub fn set_number_list_filter_value(&mut self, mut values: Vec<u16>) {
        values.sort();
        self.filter_values = values;
    }

    #[allow(clippy::comparison_chain)]
    pub fn is_match_with_attribute_value(
        &self,
        value: &ProfileAttributeValue,
        attribute_info: &Attribute,
    ) -> bool {
        if self.id != value.id {
            return false;
        }

        if attribute_info.mode.is_bitflag_mode() {
            self.as_bitflags() & value.as_bitflags() != 0
        } else if attribute_info.mode.is_number_list() {
            // Assume that both number lists are sorted
            let mut value_iter = value.as_number_list().iter();
            let mut found = true;

            for filter_number in self.as_number_list() {
                while found {
                    match value_iter.next() {
                        Some(value_number) => {
                            if value_number < filter_number {
                                // Can be found still
                                continue;
                            } else if value_number == filter_number {
                                // Found
                                break;
                            } else {
                                // Not found
                                found = false;
                                break;
                            }
                        }
                        None => {
                            found = false;
                            break;
                        }
                    }
                }
            }

            found
        } else {
            if let Some(top_level_id) = self.as_top_level_id() {
                if top_level_id == value.as_top_level_id() {
                    match self.as_sub_level_id() {
                        wanted @ Some(_) => wanted == value.as_sub_level_id(),
                        None => true,
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}

/// Public profile info
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    /// Profile text support is disabled for now.
    pub profile_text: String,
    #[schema(value_type = i64)]
    pub age: ProfileAge,
    pub attributes: Vec<ProfileAttributeValue>,
}

impl Profile {
    pub fn new(
        value: ProfileInternal,
        attributes: Vec<ProfileAttributeValue>,
    ) -> Self {
        Self {
            name: value.name,
            profile_text: value.profile_text,
            age: value.age,
            attributes,
        }
    }
}

pub struct ProfileAndProfileVersion {
    pub profile: Profile,
    pub version: ProfileVersion,
}

/// Private profile related database data
#[derive(Debug, Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::profile_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct ProfileStateInternal {
    pub search_age_range_min: ProfileAge,
    pub search_age_range_max: ProfileAge,
    #[diesel(deserialize_as = i64, serialize_as = i64)]
    pub search_group_flags: SearchGroupFlags,
    pub profile_attributes_sync_version: ProfileAttributesSyncVersion,
}

sync_version_wrappers!(ProfileAttributesSyncVersion,);

/// Subset of ProfileStateInternal which is cached in memory.
#[derive(Debug, Clone, Copy)]
pub struct ProfileStateCached {
    pub search_age_range_min: ProfileAge,
    pub search_age_range_max: ProfileAge,
    pub search_group_flags: SearchGroupFlags,
}

impl From<ProfileStateInternal> for ProfileStateCached {
    fn from(value: ProfileStateInternal) -> Self {
        Self {
            search_age_range_min: value.search_age_range_min,
            search_age_range_max: value.search_age_range_max,
            search_group_flags: value.search_group_flags,
        }
    }
}

/// Profile age value which is in inclusive range `[18, 99]`.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Eq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Integer)]
#[serde(try_from = "i64")]
#[serde(into = "i64")]
pub struct ProfileAge {
    value: u8,
}

impl ProfileAge {
    pub const MIN_AGE: u8 = 18;
    pub const MAX_AGE: u8 = 99;

    pub fn new_clamped(age: u8) -> Self {
        Self {
            value: age.clamp(Self::MIN_AGE, Self::MAX_AGE),
        }
    }
    pub fn value(&self) -> u8 {
        self.value
    }
}

impl Default for ProfileAge {
    fn default() -> Self {
        Self {
            value: Self::MIN_AGE,
        }
    }
}

impl TryFrom<i64> for ProfileAge {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < Self::MIN_AGE as i64 || value > Self::MAX_AGE as i64 {
            Err(format!(
                "Profile age must be in range [{}, {}]",
                Self::MIN_AGE,
                Self::MAX_AGE
            ))
        } else {
            Ok(Self { value: value as u8 })
        }
    }
}

impl From<ProfileAge> for i64 {
    fn from(value: ProfileAge) -> Self {
        value.value as i64
    }
}

diesel_i64_struct_try_from!(ProfileAge);

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileSearchAgeRange {
    /// Min value for this field is 18.
    pub min: u8,
    /// Max value for this field is 99.
    pub max: u8,
}

impl From<ProfileStateInternal> for ProfileSearchAgeRange {
    fn from(value: ProfileStateInternal) -> Self {
        Self {
            min: value.search_age_range_min.value(),
            max: value.search_age_range_max.value(),
        }
    }
}

/// Profile search age range which min and max are in
/// inclusive range of `[18, 99]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProfileSearchAgeRangeValidated {
    min: ProfileAge,
    max: ProfileAge,
}

impl ProfileSearchAgeRangeValidated {
    /// New range from two values. Automatically orders the values.
    pub fn new(value1: ProfileAge, value2: ProfileAge) -> Self {
        if value1.value() <= value2.value() {
            Self {
                min: value1,
                max: value2,
            }
        } else {
            Self {
                min: value2,
                max: value1,
            }
        }
    }

    pub fn min(&self) -> ProfileAge {
        self.min
    }

    pub fn max(&self) -> ProfileAge {
        self.max
    }

    pub fn is_match(&self, age: ProfileAge) -> bool {
        age.value() >= self.min.value() && age.value() <= self.max.value()
    }
}

impl TryFrom<ProfileSearchAgeRange> for ProfileSearchAgeRangeValidated {
    type Error = String;

    fn try_from(value: ProfileSearchAgeRange) -> Result<Self, Self::Error> {
        if value.min > value.max {
            Err("Min value must be less than or equal to max value".to_string())
        } else {
            let min = (value.min as i64).try_into()?;
            let max = (value.max as i64).try_into()?;
            Ok(Self { min, max })
        }
    }
}

/// My gender and what gender I'm searching for.
///
/// Fileds should be read "I'm x and I'm searching for y".
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq, Default)]
pub struct SearchGroups {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")] // Skips false
    #[schema(default = false)]
    pub man_for_woman: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub man_for_man: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub man_for_non_binary: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub woman_for_man: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub woman_for_woman: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub woman_for_non_binary: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub non_binary_for_man: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub non_binary_for_woman: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub non_binary_for_non_binary: bool,
}

impl SearchGroups {
    fn to_validated_man(self) -> Option<ValidatedSearchGroups> {
        if self.man_for_woman || self.man_for_man || self.man_for_non_binary {
            Some(ValidatedSearchGroups::ManFor {
                woman: self.man_for_woman,
                man: self.man_for_man,
                non_binary: self.man_for_non_binary,
            })
        } else {
            None
        }
    }

    fn to_validated_woman(self) -> Option<ValidatedSearchGroups> {
        if self.woman_for_man || self.woman_for_woman || self.woman_for_non_binary {
            Some(ValidatedSearchGroups::WomanFor {
                man: self.woman_for_man,
                woman: self.woman_for_woman,
                non_binary: self.woman_for_non_binary,
            })
        } else {
            None
        }
    }

    fn to_validated_non_binary(self) -> Option<ValidatedSearchGroups> {
        if self.non_binary_for_man || self.non_binary_for_woman || self.non_binary_for_non_binary {
            Some(ValidatedSearchGroups::NonBinaryFor {
                man: self.non_binary_for_man,
                woman: self.non_binary_for_woman,
                non_binary: self.non_binary_for_non_binary,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidatedSearchGroups {
    ManFor {
        woman: bool,
        man: bool,
        non_binary: bool,
    },
    WomanFor {
        man: bool,
        woman: bool,
        non_binary: bool,
    },
    NonBinaryFor {
        man: bool,
        woman: bool,
        non_binary: bool,
    },
}

impl TryFrom<SearchGroups> for ValidatedSearchGroups {
    type Error = &'static str;

    fn try_from(value: SearchGroups) -> Result<Self, Self::Error> {
        match (
            value.to_validated_man(),
            value.to_validated_woman(),
            value.to_validated_non_binary(),
        ) {
            (Some(v), None, None) => Ok(v),
            (None, Some(v), None) => Ok(v),
            (None, None, Some(v)) => Ok(v),
            (None, None, None) => Err("Gender not set"),
            _ => Err("Unambiguous gender"),
        }
    }
}

bitflags::bitflags! {
    /// Same as SearchGroups but as bitflags. The biflags are used in database.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct SearchGroupFlags: u16 {
        const MAN_FOR_WOMAN = 0x1;
        const MAN_FOR_MAN = 0x2;
        const MAN_FOR_NON_BINARY = 0x4;
        const WOMAN_FOR_MAN = 0x8;
        const WOMAN_FOR_WOMAN = 0x10;
        const WOMAN_FOR_NON_BINARY = 0x20;
        const NON_BINARY_FOR_MAN = 0x40;
        const NON_BINARY_FOR_WOMAN = 0x80;
        const NON_BINARY_FOR_NON_BINARY = 0x100;
    }
}

impl SearchGroupFlags {
    pub fn to_filter(&self) -> SearchGroupFlagsFilter {
        SearchGroupFlagsFilter::new(*self)
    }
}

impl TryFrom<i64> for SearchGroupFlags {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = TryInto::<u16>::try_into(value).map_err(|e| e.to_string())?;
        Self::from_bits(value).ok_or_else(|| "Unknown bitflag".to_string())
    }
}

impl From<SearchGroupFlags> for i64 {
    fn from(value: SearchGroupFlags) -> Self {
        value.bits() as i64
    }
}

impl From<ValidatedSearchGroups> for SearchGroupFlags {
    fn from(value: ValidatedSearchGroups) -> Self {
        let mut flags: SearchGroupFlags = Self::empty();
        match value {
            ValidatedSearchGroups::ManFor {
                woman,
                man,
                non_binary,
            } => {
                if woman {
                    flags |= Self::MAN_FOR_WOMAN;
                }
                if man {
                    flags |= Self::MAN_FOR_MAN;
                }
                if non_binary {
                    flags |= Self::MAN_FOR_NON_BINARY;
                }
            }
            ValidatedSearchGroups::WomanFor {
                man,
                woman,
                non_binary,
            } => {
                if man {
                    flags |= Self::WOMAN_FOR_MAN;
                }
                if woman {
                    flags |= Self::WOMAN_FOR_WOMAN;
                }
                if non_binary {
                    flags |= Self::WOMAN_FOR_NON_BINARY;
                }
            }
            ValidatedSearchGroups::NonBinaryFor {
                man,
                woman,
                non_binary,
            } => {
                if man {
                    flags |= Self::NON_BINARY_FOR_MAN;
                }
                if woman {
                    flags |= Self::NON_BINARY_FOR_WOMAN;
                }
                if non_binary {
                    flags |= Self::NON_BINARY_FOR_NON_BINARY;
                }
            }
        }
        flags
    }
}

impl From<SearchGroupFlags> for SearchGroups {
    fn from(v: SearchGroupFlags) -> Self {
        Self {
            man_for_woman: v.contains(SearchGroupFlags::MAN_FOR_WOMAN),
            man_for_man: v.contains(SearchGroupFlags::MAN_FOR_MAN),
            man_for_non_binary: v.contains(SearchGroupFlags::MAN_FOR_NON_BINARY),
            woman_for_man: v.contains(SearchGroupFlags::WOMAN_FOR_MAN),
            woman_for_woman: v.contains(SearchGroupFlags::WOMAN_FOR_WOMAN),
            woman_for_non_binary: v.contains(SearchGroupFlags::WOMAN_FOR_NON_BINARY),
            non_binary_for_man: v.contains(SearchGroupFlags::NON_BINARY_FOR_MAN),
            non_binary_for_woman: v.contains(SearchGroupFlags::NON_BINARY_FOR_WOMAN),
            non_binary_for_non_binary: v.contains(SearchGroupFlags::NON_BINARY_FOR_NON_BINARY),
        }
    }
}

/// Filter which finds matches with other SearchGroupFlags.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SearchGroupFlagsFilter {
    filter: SearchGroupFlags,
}

impl SearchGroupFlagsFilter {
    fn new(flags: SearchGroupFlags) -> Self {
        let mut filter = SearchGroupFlags::empty();

        // Man
        if flags.contains(SearchGroupFlags::MAN_FOR_WOMAN) {
            filter |= SearchGroupFlags::WOMAN_FOR_MAN;
        }
        if flags.contains(SearchGroupFlags::MAN_FOR_MAN) {
            filter |= SearchGroupFlags::MAN_FOR_MAN;
        }
        if flags.contains(SearchGroupFlags::MAN_FOR_NON_BINARY) {
            filter |= SearchGroupFlags::NON_BINARY_FOR_MAN;
        }
        // Woman
        if flags.contains(SearchGroupFlags::WOMAN_FOR_MAN) {
            filter |= SearchGroupFlags::MAN_FOR_WOMAN;
        }
        if flags.contains(SearchGroupFlags::WOMAN_FOR_WOMAN) {
            filter |= SearchGroupFlags::WOMAN_FOR_WOMAN;
        }
        if flags.contains(SearchGroupFlags::WOMAN_FOR_NON_BINARY) {
            filter |= SearchGroupFlags::NON_BINARY_FOR_WOMAN;
        }
        // Non-binary
        if flags.contains(SearchGroupFlags::NON_BINARY_FOR_MAN) {
            filter |= SearchGroupFlags::MAN_FOR_NON_BINARY;
        }
        if flags.contains(SearchGroupFlags::NON_BINARY_FOR_WOMAN) {
            filter |= SearchGroupFlags::WOMAN_FOR_NON_BINARY;
        }
        if flags.contains(SearchGroupFlags::NON_BINARY_FOR_NON_BINARY) {
            filter |= SearchGroupFlags::NON_BINARY_FOR_NON_BINARY;
        }

        Self { filter }
    }

    fn is_match(&self, flags: SearchGroupFlags) -> bool {
        self.filter.intersects(flags)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct ProfileUpdate {
    /// This must be empty because profile text support is disabled.
    pub profile_text: String,
    pub name: String,
    #[schema(value_type = i64)]
    pub age: ProfileAge,
    pub attributes: Vec<ProfileAttributeValueUpdate>,
}

impl ProfileUpdate {
    pub fn validate(
        mut self,
        attribute_info: Option<&ProfileAttributes>,
    ) -> Result<ProfileUpdateValidated, String> {
        let mut hash_set = HashSet::new();
        for a in &mut self.attributes {
            if !hash_set.insert(a.id) {
                return Err("Duplicate attribute ID".to_string());
            }

            if let Some(info) = attribute_info {
                let attribute_info = info.attributes.get(a.id as usize);
                match attribute_info {
                    None => return Err("Unknown attribute ID".to_string()),
                    Some(info) => {
                        if info.mode.is_number_list() && a.values.len() > NUMBER_LIST_ATTRIBUTE_MAX_VALUES {
                            return Err(format!("Number list attribute supports max {} values", NUMBER_LIST_ATTRIBUTE_MAX_VALUES));
                        }

                        if info.mode.is_number_list() {
                            a.values.sort();
                        }
                    }
                }
            } else {
                return Err("Profile attributes are disabled".to_string());
            }
        }

        if !self.profile_text.is_empty() {
            return Err("Profile text is not empty".to_string());
        }

        Ok(ProfileUpdateValidated {
            profile_text: self.profile_text,
            name: self.name,
            age: self.age,
            attributes: self.attributes,
        })
    }
}

/// Makes sure that the number list attributes are sorted.
#[derive(Debug, Clone, Default)]
pub struct ProfileUpdateValidated {
    pub profile_text: String,
    pub name: String,
    pub age: ProfileAge,
    pub attributes: Vec<ProfileAttributeValueUpdate>,
}

impl ProfileUpdateValidated {
    pub fn equals_with(&self, other: &Profile) -> bool {
        let basic = self.name == other.name
            && self.profile_text == other.profile_text
            && self.age == other.age;
        if basic {
            let a1: HashMap<u16, ProfileAttributeValueUpdate> =
                HashMap::from_iter(self.attributes.iter().map(|v| (v.id, v.clone())));
            let a2: HashMap<u16, ProfileAttributeValueUpdate> =
                HashMap::from_iter(other.attributes.iter().map(|v| (v.id, v.clone().into())));

            a1 == a2
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfileUpdateInternal {
    pub new_data: ProfileUpdateValidated,
    /// Version used for caching profile in client side.
    pub version: ProfileVersion,
}

impl ProfileUpdateInternal {
    pub fn new(new_data: ProfileUpdateValidated) -> Self {
        Self {
            new_data,
            version: ProfileVersion::new_random(),
        }
    }
}

// TODO: Create ProfileInternal and have all attributes there.

// #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
// pub struct ProfileInternal {
//     profile: Profile,
//     /// Profile visibility. Set true to make profile public.
//     public: Option<bool>,
// }

// impl ProfileInternal {
//     pub fn new(name: String) -> Self {
//         Self {
//             profile: Profile::new(name),
//             public: None,
//         }
//     }

//     pub fn profile(&self) -> &Profile {
//         &self.profile
//     }

//     pub fn public(&self) -> bool {
//         self.public.unwrap_or_default()
//     }
// }

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Default)]
#[serde(try_from = "f64")]
#[serde(into = "f64")]
pub struct FiniteDouble {
    value: f64,
}

impl TryFrom<f64> for FiniteDouble {
    type Error = String;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self { value })
        } else {
            Err("Value must be finite".to_string())
        }
    }
}

impl From<FiniteDouble> for f64 {
    fn from(value: FiniteDouble) -> Self {
        value.value
    }
}

/// Location in latitude and longitude.
/// The values are not NaN, infinity or negative infinity.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Default,
    Queryable,
    Selectable,
    AsChangeset,
)]
#[diesel(table_name = crate::schema::profile_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct Location {
    #[schema(value_type = f64)]
    #[diesel(deserialize_as = f64, serialize_as = f64)]
    latitude: FiniteDouble,
    #[schema(value_type = f64)]
    #[diesel(deserialize_as = f64, serialize_as = f64)]
    longitude: FiniteDouble,
}

impl Location {
    pub fn latitude(&self) -> f64 {
        self.latitude.into()
    }

    pub fn longitude(&self) -> f64 {
        self.longitude.into()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct FavoriteProfilesPage {
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct ProfilePage {
    pub profiles: Vec<ProfileLink>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ProfileLink {
    id: AccountId,
    version: ProfileVersion,
    /// This is optional because media component owns it.
    content_version: Option<ProfileContentVersion>,
}

impl ProfileLink {
    pub(crate) fn new(
        id: AccountId,
        profile: &ProfileInternal,
        content_version: Option<ProfileContentVersion>,
    ) -> Self {
        Self {
            id,
            version: profile.version_uuid,
            content_version,
        }
    }
}

#[derive(Debug)]
pub struct ProfileQueryMakerDetails {
    pub age: ProfileAge,
    pub search_age_range: ProfileSearchAgeRangeValidated,
    pub search_groups_filter: SearchGroupFlagsFilter,
    pub attribute_filters: Vec<ProfileAttributeFilterValue>,
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
        }
    }
}

/// All data which location index needs for returning filtered profiles when
/// user queries new profiles.
#[derive(Debug, Clone, PartialEq)]
pub struct LocationIndexProfileData {
    /// Simple profile information for client.
    profile_link: ProfileLink,
    age: ProfileAge,
    search_age_range: ProfileSearchAgeRangeValidated,
    search_groups: SearchGroupFlags,
    attributes: SortedProfileAttributes,
}

impl LocationIndexProfileData {
    pub fn new(
        id: AccountId,
        profile: &ProfileInternal,
        state: &ProfileStateCached,
        attributes: SortedProfileAttributes,
        profile_content_version: Option<ProfileContentVersion>,
    ) -> Self {
        Self {
            profile_link: ProfileLink::new(id, profile, profile_content_version),
            age: profile.age,
            search_age_range: ProfileSearchAgeRangeValidated::new(
                state.search_age_range_min,
                state.search_age_range_max,
            ),
            search_groups: state.search_group_flags,
            attributes,
        }
    }

    pub fn is_match(
        &self,
        query_maker_details: &ProfileQueryMakerDetails,
        attribute_info: Option<&ProfileAttributes>,
    ) -> bool {
        let mut is_match = self.search_age_range.is_match(query_maker_details.age)
            && query_maker_details.search_age_range.is_match(self.age)
            && query_maker_details
                .search_groups_filter
                .is_match(self.search_groups);

        if let Some(attribute_info) = attribute_info {
            is_match &= self.attribute_filters_match(query_maker_details, attribute_info);
        }

        is_match
    }

    fn attribute_filters_match(
        &self,
        query_maker_details: &ProfileQueryMakerDetails,
        attribute_info: &ProfileAttributes,
    ) -> bool {
        for filter in &query_maker_details.attribute_filters {
            let attribute_info =
                if let Some(info) = attribute_info.attributes.get(filter.id as usize) {
                    info
                } else {
                    return false;
                };

            if let Some(value) = self.attributes.find_id(filter.id) {
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

impl From<&LocationIndexProfileData> for ProfileLink {
    fn from(value: &LocationIndexProfileData) -> Self {
        value.profile_link
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    IntoParams,
    PartialEq,
    Eq,
    Hash,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Binary)]
pub struct ProfileVersion {
    version: uuid::Uuid,
}

impl ProfileVersion {
    pub(crate) fn new(version: uuid::Uuid) -> Self {
        Self { version }
    }

    pub fn new_random() -> Self {
        let version = uuid::Uuid::new_v4();
        Self { version }
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.version
    }
}

diesel_uuid_wrapper!(ProfileVersion);

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetProfileQueryParam {
    /// Profile version UUID
    version: Option<uuid::Uuid>,
    /// If requested profile is not public, allow getting the profile
    /// data if the requested profile is a match.
    #[serde(default)]
    is_match: bool,
}

impl GetProfileQueryParam {
    pub fn profile_version(self) -> Option<ProfileVersion> {
        self.version.map(ProfileVersion::new)
    }

    pub fn allow_get_profile_if_match(self) -> bool {
        self.is_match
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetProfileResult {
    /// Profile data if it is newer than the version in the query.
    pub profile: Option<Profile>,
    /// If empty then profile does not exist or current account does
    /// not have access to the profile.
    pub version: Option<ProfileVersion>,
}

impl GetProfileResult {
    pub fn profile_with_version_response(info: ProfileAndProfileVersion) -> Self {
        Self {
            profile: Some(info.profile),
            version: Some(info.version),
        }
    }

    pub fn current_version_latest_response(version: ProfileVersion) -> Self {
        Self {
            profile: None,
            version: Some(version),
        }
    }

    pub fn empty() -> Self {
        Self {
            profile: None,
            version: None,
        }
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

    pub fn next_down(&self) -> usize {
        self.next_down.load(Ordering::Relaxed) as usize
    }

    pub fn next_up(&self) -> usize {
        self.next_up.load(Ordering::Relaxed) as usize
    }

    pub fn next_left(&self) -> usize {
        self.next_left.load(Ordering::Relaxed) as usize
    }

    pub fn next_right(&self) -> usize {
        self.next_right.load(Ordering::Relaxed) as usize
    }

    pub fn profiles(&self) -> bool {
        self.profiles_in_this_area.load(Ordering::Relaxed)
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
