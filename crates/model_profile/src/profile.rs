use std::collections::{HashMap, HashSet};

use diesel::{AsExpression, FromSqlRow, prelude::*, sql_types::BigInt};
use model::{AttributeId, ProfileAge};
use model_server_data::{
    LastSeenTime, LastSeenTimeFilter, MaxDistanceKm, MinDistanceKm, ProfileAttributeValue,
    ProfileAttributeValueUpdate, ProfileAttributesInternal, ProfileCreatedTimeFilter,
    ProfileEditedTime, ProfileEditedTimeFilter, ProfileInternal, ProfileNameModerationState,
    ProfileStateCached, ProfileTextMaxCharactersFilter, ProfileTextMinCharactersFilter,
    ProfileTextModerationState, ProfileVersion, SearchGroupFlags, SortedProfileAttributes,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use simple_backend_model::{UnixTime, diesel_i64_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, sync_version_wrappers};

mod age;
pub use age::*;

mod available_attributes;
pub use available_attributes::*;

mod filter;
pub use filter::*;

mod iterator;
pub use iterator::*;

mod search_group;
pub use search_group::*;

mod statistics;
pub use statistics::*;

mod moderation;
pub use moderation::*;

mod report;
pub use report::*;

/// Public profile info
#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    /// Profile text support is disabled for now.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[schema(default = "")]
    pub ptext: String,
    #[schema(value_type = i64)]
    pub age: ProfileAge,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(default = json!([]))]
    pub attributes: Vec<ProfileAttributeValue>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    unlimited_likes: bool,
    /// The name has been accepted using allowlist or manual moderation.
    #[serde(skip_serializing_if = "is_true")]
    #[schema(default = true)]
    name_accepted: bool,
    /// The profile text has been accepted by bot or human moderator.
    #[serde(skip_serializing_if = "is_true")]
    #[schema(default = true)]
    ptext_accepted: bool,
}

fn is_true(value: &bool) -> bool {
    *value
}

impl Profile {
    pub fn new(
        value: ProfileInternal,
        profile_name_moderation_state: Option<ProfileNameModerationState>,
        profile_text_moderation_state: Option<ProfileTextModerationState>,
        attributes: Vec<ProfileAttributeValue>,
        unlimited_likes: bool,
    ) -> Self {
        Self {
            name: value.profile_name,
            ptext: value.profile_text,
            age: value.age,
            attributes,
            unlimited_likes,
            name_accepted: profile_name_moderation_state
                .map(|v| v.0.is_accepted())
                .unwrap_or_default(),
            ptext_accepted: profile_text_moderation_state
                .map(|v| v.0.is_accepted())
                .unwrap_or_default(),
        }
    }

    pub fn name_accepted(&self) -> bool {
        self.name_accepted
    }

    pub fn unlimited_likes(&self) -> bool {
        self.unlimited_likes
    }
}

pub struct ProfileAndProfileVersion {
    pub profile: Profile,
    pub version: ProfileVersion,
    pub last_seen_time: LastSeenTime,
}

/// Private profile related database data
#[derive(Debug, Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::profile_state)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct ProfileStateInternal {
    pub search_age_range_min: ProfileAge,
    pub search_age_range_max: ProfileAge,
    #[diesel(deserialize_as = i64, serialize_as = i64)]
    pub search_group_flags: SearchGroupFlags,
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
    pub unlimited_likes_filter: Option<bool>,
    pub min_distance_km_filter: Option<MinDistanceKm>,
    pub max_distance_km_filter: Option<MaxDistanceKm>,
    pub profile_created_time_filter: Option<ProfileCreatedTimeFilter>,
    pub profile_edited_time_filter: Option<ProfileEditedTimeFilter>,
    pub profile_text_min_characters_filter: Option<ProfileTextMinCharactersFilter>,
    pub profile_text_max_characters_filter: Option<ProfileTextMaxCharactersFilter>,
    pub random_profile_order: bool,
    pub profile_sync_version: ProfileSyncVersion,
    pub profile_edited_unix_time: ProfileEditedTime,
}

impl From<ProfileStateInternal> for ProfileStateCached {
    fn from(value: ProfileStateInternal) -> Self {
        Self {
            search_age_range_min: value.search_age_range_min,
            search_age_range_max: value.search_age_range_max,
            search_group_flags: value.search_group_flags,
            last_seen_time_filter: value.last_seen_time_filter,
            unlimited_likes_filter: value.unlimited_likes_filter,
            min_distance_km_filter: value.min_distance_km_filter,
            max_distance_km_filter: value.max_distance_km_filter,
            profile_created_time_filter: value.profile_created_time_filter,
            profile_edited_time_filter: value.profile_edited_time_filter,
            profile_text_min_characters_filter: value.profile_text_min_characters_filter,
            profile_text_max_characters_filter: value.profile_text_max_characters_filter,
            random_profile_order: value.random_profile_order,
            profile_edited_time: value.profile_edited_unix_time,
        }
    }
}

sync_version_wrappers!(ProfileSyncVersion,);

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct ProfileUpdate {
    pub ptext: String,
    pub name: String,
    #[schema(value_type = i64)]
    pub age: ProfileAge,
    pub attributes: Vec<ProfileAttributeValueUpdate>,
}

impl ProfileUpdate {
    /// `AcceptedProfileAges` is checked only if it is Some.
    pub fn validate(
        mut self,
        attribute_info: Option<&ProfileAttributesInternal>,
        profile_name_regex: Option<&Regex>,
        current_profile: &Profile,
        initial_age: Option<InitialProfileAge>,
    ) -> Result<ProfileUpdateValidated, String> {
        let mut hash_set = HashSet::new();
        for a in &mut self.attributes {
            if !hash_set.insert(a.id) {
                return Err("Duplicate attribute ID".to_string());
            }

            if let Some(info) = attribute_info {
                let attribute_info = info.get_attribute(a.id);
                match attribute_info {
                    None => return Err("Unknown attribute ID".to_string()),
                    Some(info) => {
                        let error = || {
                            Err(format!(
                                "Attribute supports max {} selected values",
                                info.max_selected,
                            ))
                        };
                        if info.mode.is_bitflag() {
                            let selected = a.v.first().copied().unwrap_or_default().count_ones();
                            if selected > info.max_selected.into() {
                                return error();
                            }
                        } else if a.v.len() > info.max_selected.into() {
                            return error();
                        }
                    }
                }
            } else {
                return Err("Profile attributes are disabled".to_string());
            }
        }

        if self.name.len() > 100 {
            return Err("Profile name is too long".to_string());
        }

        if self.name != self.name.trim() {
            return Err("Profile name is not trimmed".to_string());
        }

        if let Some(c) = self.name.chars().next() {
            if !c.is_uppercase() {
                return Err("Profile name does not start with uppercase letter".to_string());
            }
        }

        if let Some(regex) = profile_name_regex {
            if !regex.is_match(&self.name) {
                return Err("Profile name does not match with profile name regex".to_string());
            }
        }

        if self.ptext.len() > 2000 {
            return Err("Profile text is too long".to_string());
        }

        if self.ptext != self.ptext.trim() {
            return Err("Profile text is not trimmed".to_string());
        }

        if self.age != current_profile.age {
            if let Some(age_range) = initial_age {
                if !age_range.is_age_valid(self.age) {
                    return Err(
                        "The new profile age is not in the current accepted profile age range"
                            .to_string(),
                    );
                }
            }
        }

        Ok(ProfileUpdateValidated {
            ptext: self.ptext,
            name: self.name,
            age: self.age,
            attributes: self.attributes,
        })
    }
}

/// Makes sure that the number list attributes are sorted.
#[derive(Debug, Clone, Default)]
pub struct ProfileUpdateValidated {
    pub ptext: String,
    pub name: String,
    pub age: ProfileAge,
    pub attributes: Vec<ProfileAttributeValueUpdate>,
}

impl ProfileUpdateValidated {
    pub fn equals_with(&self, other: &Profile) -> bool {
        let basic = self.name == other.name && self.ptext == other.ptext && self.age == other.age;
        if basic {
            let a1: HashMap<AttributeId, ProfileAttributeValueUpdate> =
                HashMap::from_iter(self.attributes.iter().map(|v| (v.id, v.clone())));
            let a2: HashMap<AttributeId, ProfileAttributeValueUpdate> =
                HashMap::from_iter(other.attributes.iter().map(|v| (v.id(), v.clone().into())));

            a1 == a2
        } else {
            false
        }
    }

    pub fn update_to_profile(&self, target: &mut ProfileInternal) {
        target.profile_name.clone_from(&self.name);
        target.profile_text.clone_from(&self.ptext);
        target.age = self.age;
    }

    pub fn update_to_attributes(&self, target: &mut SortedProfileAttributes) {
        let attributes = self
            .attributes
            .iter()
            .filter_map(|v| ProfileAttributeValue::from_update(v.clone()))
            .collect::<Vec<_>>();
        target.set_attributes(attributes);
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct FavoriteProfilesPage {
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetProfileQueryParam {
    /// Profile version UUID
    v: Option<simple_backend_utils::UuidBase64Url>,
    /// If requested profile is not public, allow getting the profile
    /// data if the requested profile is a match.
    #[serde(default)]
    is_match: bool,
}

impl GetProfileQueryParam {
    pub fn profile_version(self) -> Option<ProfileVersion> {
        self.v.map(ProfileVersion::new_base_64_url)
    }

    pub fn allow_get_profile_if_match(self) -> bool {
        self.is_match
    }
}

#[derive(Debug, Clone, Serialize, ToSchema, IntoParams)]
pub struct GetProfileResult {
    /// Profile data if it is newer than the version in the query.
    pub p: Option<Profile>,
    /// If empty then profile does not exist or current account does
    /// not have access to the profile.
    pub v: Option<ProfileVersion>,
    lst: Option<LastSeenTime>,
}

impl GetProfileResult {
    pub fn profile_with_version_response(info: ProfileAndProfileVersion) -> Self {
        Self {
            p: Some(info.profile),
            v: Some(info.version),
            lst: Some(info.last_seen_time),
        }
    }

    pub fn current_version_latest_response(
        version: ProfileVersion,
        last_seen_time: Option<LastSeenTime>,
    ) -> Self {
        Self {
            p: None,
            v: Some(version),
            lst: last_seen_time,
        }
    }

    pub fn empty() -> Self {
        Self {
            p: None,
            v: None,
            lst: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GetMyProfileResult {
    pub p: Profile,
    pub v: ProfileVersion,
    pub sv: ProfileSyncVersion,
    pub lst: Option<LastSeenTime>,
    pub name_moderation_info: Option<ProfileStringModerationInfo>,
    pub text_moderation_info: Option<ProfileStringModerationInfo>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct InitialProfileAge {
    #[schema(value_type = i64)]
    pub initial_profile_age: ProfileAge,
    pub initial_profile_age_set_unix_time: UnixTime,
}

impl InitialProfileAge {
    pub fn is_age_valid(&self, age: ProfileAge) -> bool {
        if age.value() < self.initial_profile_age.value() {
            return false;
        }

        let current_time = UnixTime::current_time();
        match (
            current_time.year(),
            self.initial_profile_age_set_unix_time.year(),
        ) {
            (Some(current_year), Some(initial_year)) => {
                let initial_age: i32 = self.initial_profile_age.value().into();
                let year_diff = current_year - initial_year;
                let min = initial_age + year_diff - 1;
                let max = initial_age + year_diff + 1;
                let age: i32 = age.value().into();
                min <= age && age <= max
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct GetInitialProfileAgeResult {
    pub value: Option<InitialProfileAge>,
}
