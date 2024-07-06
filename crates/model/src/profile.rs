use std::collections::{HashMap, HashSet};

use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_wrapper, diesel_uuid_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{
    sync_version_wrappers, AccountId, AccountIdDb, SyncVersion, SyncVersionUtils, ProfileContentVersion,
};

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

mod location;
pub use location::*;

mod search_group;
pub use search_group::*;

mod last_seen_time;
pub use last_seen_time::*;

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

/// Public profile info
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    /// Profile text support is disabled for now.
    pub profile_text: String,
    #[schema(value_type = i64)]
    pub age: ProfileAge,
    pub attributes: Vec<ProfileAttributeValue>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    unlimited_likes: bool,
}

impl Profile {
    pub fn new(
        value: ProfileInternal,
        attributes: Vec<ProfileAttributeValue>,
        unlimited_likes: bool,
    ) -> Self {
        Self {
            name: value.name,
            profile_text: value.profile_text,
            age: value.age,
            attributes,
            unlimited_likes,
        }
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
pub struct ProfileStateInternal {
    pub search_age_range_min: ProfileAge,
    pub search_age_range_max: ProfileAge,
    #[diesel(deserialize_as = i64, serialize_as = i64)]
    pub search_group_flags: SearchGroupFlags,
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
    pub profile_attributes_sync_version: ProfileAttributesSyncVersion,
}

sync_version_wrappers!(ProfileAttributesSyncVersion,);

/// Subset of ProfileStateInternal which is cached in memory.
#[derive(Debug, Clone, Copy)]
pub struct ProfileStateCached {
    pub search_age_range_min: ProfileAge,
    pub search_age_range_max: ProfileAge,
    pub search_group_flags: SearchGroupFlags,
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
}

impl From<ProfileStateInternal> for ProfileStateCached {
    fn from(value: ProfileStateInternal) -> Self {
        Self {
            search_age_range_min: value.search_age_range_min,
            search_age_range_max: value.search_age_range_max,
            search_group_flags: value.search_group_flags,
            last_seen_time_filter: value.last_seen_time_filter,
        }
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
    /// If the last seen time is not None, then it is Unix timestamp or -1 if
    /// the profile is currently online.
    last_seen_time: Option<LastSeenTime>,
}

impl ProfileLink {
    pub(crate) fn new(
        id: AccountId,
        profile: &ProfileInternal,
        content_version: Option<ProfileContentVersion>,
        last_seen_time: Option<LastSeenTime>,
    ) -> Self {
        Self {
            id,
            version: profile.version_uuid,
            content_version,
            last_seen_time,
        }
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
    last_seen_time: Option<LastSeenTime>,
}

impl GetProfileResult {
    pub fn profile_with_version_response(
        info: ProfileAndProfileVersion,
    ) -> Self {
        Self {
            profile: Some(info.profile),
            version: Some(info.version),
            last_seen_time: info.last_seen_time,
        }
    }

    pub fn current_version_latest_response(
        version: ProfileVersion,
        last_seen_time: Option<LastSeenTime>,
    ) -> Self {
        Self {
            profile: None,
            version: Some(version),
            last_seen_time,
        }
    }

    pub fn empty() -> Self {
        Self {
            profile: None,
            version: None,
            last_seen_time: None,
        }
    }
}
