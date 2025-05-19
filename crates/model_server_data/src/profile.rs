use diesel::{prelude::Queryable, sql_types::Binary, Selectable};
use model::{AccountIdDb, ProfileAge};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_uuid_wrapper;
use utoipa::{IntoParams, ToSchema};

mod attribute;
pub use attribute::*;

mod available_attributes;
pub use available_attributes::*;

mod age;
pub use age::*;

mod index;
pub use index::*;

mod last_time_seen;
pub use last_time_seen::*;

mod profile_created_time;
pub use profile_created_time::*;

mod profile_edited_time;
pub use profile_edited_time::*;

mod iterator;
pub use iterator::*;

mod search_groups;
pub use search_groups::*;

mod name;
pub use name::*;

mod text;
pub use text::*;

mod filter;
pub use filter::*;

mod location;
pub use location::*;

mod statistics;
pub use statistics::*;

mod search;
pub use search::*;

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
    v: simple_backend_utils::UuidBase64Url,
}

impl ProfileVersion {
    pub fn new_base_64_url(version: simple_backend_utils::UuidBase64Url) -> Self {
        Self { v: version }
    }

    fn diesel_uuid_wrapper_new(v: simple_backend_utils::UuidBase64Url) -> Self {
        Self { v }
    }

    pub fn new_random() -> Self {
        Self {
            v: simple_backend_utils::UuidBase64Url::new_random_id(),
        }
    }

    fn diesel_uuid_wrapper_as_uuid(&self) -> &simple_backend_utils::UuidBase64Url {
        &self.v
    }
}

diesel_uuid_wrapper!(ProfileVersion);

/// Subset of ProfileStateInternal which is cached in memory.
#[derive(Debug, Clone, Copy)]
pub struct ProfileStateCached {
    pub search_age_range_min: ProfileAge,
    pub search_age_range_max: ProfileAge,
    pub search_group_flags: SearchGroupFlags,
    pub last_seen_time_filter: Option<LastSeenTimeFilter>,
    pub unlimited_likes_filter: Option<bool>,
    pub profile_created_time_filter: Option<ProfileCreatedTimeFilter>,
    pub profile_edited_time_filter: Option<ProfileEditedTimeFilter>,
    pub profile_text_min_characters_filter: Option<ProfileTextMinCharactersFilter>,
    pub profile_text_max_characters_filter: Option<ProfileTextMaxCharactersFilter>,
    pub max_distance_km_filter: Option<MaxDistanceKm>,
    pub random_profile_order: bool,
    pub profile_name_moderation_state: ProfileNameModerationState,
    pub profile_text_moderation_state: ProfileTextModerationState,
    pub profile_edited_time: ProfileEditedTime,
}
