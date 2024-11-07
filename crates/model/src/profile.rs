use std::collections::{HashMap, HashSet};

use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_wrapper, diesel_uuid_wrapper, UnixTime};
use utoipa::{IntoParams, ToSchema};

use crate::{sync_version_wrappers, AccountId, AccountIdDb, SyncVersion, SyncVersionUtils};

mod age;
pub use age::*;

mod attribute;
pub use attribute::*;

mod available_attributes;
pub use available_attributes::*;

mod filter;
pub use filter::*;

mod index;
pub use index::*;

mod iterator;
pub use iterator::*;

mod location;
pub use location::*;

mod search_group;
pub use search_group::*;

mod last_seen_time;
pub use last_seen_time::*;

mod statistics;
pub use statistics::*;

mod text;
pub use text::*;

const NUMBER_LIST_ATTRIBUTE_MAX_VALUES: usize = 8;

/// Profile's database data
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::profile)]
#[diesel(check_for_backend(crate::Db))]
pub struct ProfileInternal {
    pub account_id: AccountIdDb,
    pub version_uuid: ProfileVersion,
    pub name: String,
    pub name_accepted: bool,
    pub profile_text: String,
    pub age: ProfileAge,
}

impl ProfileInternal {
    pub fn update_from(&mut self, update: &ProfileUpdateValidated) {
        self.name.clone_from(&update.name);
        self.profile_text.clone_from(&update.ptext);
        self.age = update.age;
    }
}

/// Public profile info
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
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
    #[serde(default = "name_accepted_default", skip_serializing_if = "is_true")]
    #[schema(default = true)]
    name_accepted: bool,
    /// The profile text has been accepted by bot or human moderator.
    #[serde(default = "ptext_accepted_default", skip_serializing_if = "is_true")]
    #[schema(default = true)]
    ptext_accepted: bool,
}

fn name_accepted_default() -> bool {
    true
}

fn ptext_accepted_default() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

impl Profile {
    pub fn new(
        value: ProfileInternal,
        profile_text_moderation_state: ProfileTextModerationState,
        attributes: Vec<ProfileAttributeValue>,
        unlimited_likes: bool,
    ) -> Self {
        Self {
            name: value.name,
            ptext: value.profile_text,
            age: value.age,
            attributes,
            unlimited_likes,
            name_accepted: value.name_accepted,
            ptext_accepted: profile_text_moderation_state.is_accepted(),
        }
    }

    pub fn name_accepted(&self) -> bool {
        self.name_accepted
    }
}

pub struct ProfileAndProfileVersion {
    pub profile: Profile,
    pub version: ProfileVersion,
    pub last_seen_time: Option<LastSeenTime>,
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
    pub profile_attributes_sync_version: ProfileAttributesSyncVersion,
    pub profile_sync_version: ProfileSyncVersion,
    pub profile_name_denied: bool,
    pub profile_text_moderation_state: ProfileTextModerationState,
    pub profile_text_moderation_rejected_reason_category: Option<ProfileTextModerationRejectedReasonCategory>,
    pub profile_text_moderation_rejected_reason_details: Option<ProfileTextModerationRejectedReasonDetails>,
    pub profile_text_moderation_moderator_account_id: Option<AccountIdDb>,
}

sync_version_wrappers!(ProfileAttributesSyncVersion, ProfileSyncVersion,);

/// Subset of ProfileStateInternal which is cached in memory.
#[derive(Debug, Clone, Copy)]
pub struct ProfileStateCached {
    pub search_age_range_min: ProfileAge,
    pub search_age_range_max: ProfileAge,
    pub search_group_flags: SearchGroupFlags,
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
    pub unlimited_likes_filter: Option<bool>,
    pub profile_text_moderation_state: ProfileTextModerationState,
}

impl From<ProfileStateInternal> for ProfileStateCached {
    fn from(value: ProfileStateInternal) -> Self {
        Self {
            search_age_range_min: value.search_age_range_min,
            search_age_range_max: value.search_age_range_max,
            search_group_flags: value.search_group_flags,
            last_seen_time_filter: value.last_seen_time_filter,
            unlimited_likes_filter: value.unlimited_likes_filter,
            profile_text_moderation_state: value.profile_text_moderation_state,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct ProfileUpdate {
    /// This must be empty because profile text support is disabled.
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
        attribute_info: Option<&ProfileAttributes>,
        current_profile: &Profile,
        accepted_profile_ages: Option<AcceptedProfileAges>,
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
                        if info.mode.is_number_list() && a.v.len() > NUMBER_LIST_ATTRIBUTE_MAX_VALUES {
                            return Err(format!("Number list attribute supports max {} values", NUMBER_LIST_ATTRIBUTE_MAX_VALUES));
                        }

                        if info.mode.is_number_list() {
                            a.v.sort();
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

        if self.ptext.len() > 400 {
            return Err("Profile text is too long".to_string());
        }

        if self.ptext != self.ptext.trim() {
            return Err("Profile text is not trimmed".to_string());
        }

        if self.age != current_profile.age {
            if let Some(age_range) = accepted_profile_ages {
                if !age_range.is_age_valid(self.age) {
                    return Err("The new profile age is not in the current accepted profile age range".to_string());
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
        let basic = self.name == other.name
            && self.ptext == other.ptext
            && self.age == other.age;
        if basic {
            let a1: HashMap<u16, ProfileAttributeValueUpdate> =
                HashMap::from_iter(self.attributes.iter().map(|v| (v.id, v.clone())));
            let a2: HashMap<u16, ProfileAttributeValueUpdate> =
                HashMap::from_iter(other.attributes.iter().map(|v| (v.id(), v.clone().into())));

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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct FavoriteProfilesPage {
    pub profiles: Vec<AccountId>,
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
    v: uuid::Uuid,
}

impl ProfileVersion {
    pub(crate) fn new(version: uuid::Uuid) -> Self {
        Self { v: version }
    }

    pub fn new_random() -> Self {
        let version = uuid::Uuid::new_v4();
        Self { v: version }
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.v
    }
}

diesel_uuid_wrapper!(ProfileVersion);

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetProfileQueryParam {
    /// Profile version UUID
    v: Option<uuid::Uuid>,
    /// If requested profile is not public, allow getting the profile
    /// data if the requested profile is a match.
    #[serde(default)]
    is_match: bool,
}

impl GetProfileQueryParam {
    pub fn profile_version(self) -> Option<ProfileVersion> {
        self.v.map(ProfileVersion::new)
    }

    pub fn allow_get_profile_if_match(self) -> bool {
        self.is_match
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetProfileResult {
    /// Profile data if it is newer than the version in the query.
    pub p: Option<Profile>,
    /// If empty then profile does not exist or current account does
    /// not have access to the profile.
    pub v: Option<ProfileVersion>,
    lst: Option<LastSeenTime>,
}

impl GetProfileResult {
    pub fn profile_with_version_response(
        info: ProfileAndProfileVersion,
    ) -> Self {
        Self {
            p: Some(info.profile),
            v: Some(info.version),
            lst: info.last_seen_time,
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetMyProfileResult {
    pub p: Profile,
    pub v: ProfileVersion,
    pub sv: ProfileSyncVersion,
    pub lst: Option<LastSeenTime>,
    pub text_moderation_info: ProfileTextModerationInfo,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct AcceptedProfileAges {
    #[schema(value_type = i64)]
    pub profile_initial_age: ProfileAge,
    pub profile_initial_age_set_unix_time: UnixTime,
}

impl AcceptedProfileAges {
    pub fn is_age_valid(&self, age: ProfileAge) -> bool {
        if age.value() < self.profile_initial_age.value() {
            return false;
        }

        let current_time = UnixTime::current_time();
        match (current_time.year(), self.profile_initial_age_set_unix_time.year()) {
            (Some(current_year), Some(initial_year)) => {
                let initial_age: i32 = self.profile_initial_age.value().into();
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
pub struct GetInitialProfileAgeInfoResult {
    pub info: Option<AcceptedProfileAges>,
}
