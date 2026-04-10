use diesel::{Selectable, prelude::Queryable};
use model::{AccountIdDb, ProfileAge, ProfileVersion};
use simple_backend_model::NonEmptyString;

mod attribute;
pub use attribute::*;

mod attributes_schema;
pub use attributes_schema::*;

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

mod moderation;
pub use moderation::*;

mod text;
pub use text::*;

mod verification;
pub use verification::*;

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
    pub profile_name: Option<NonEmptyString>,
    pub profile_text: Option<NonEmptyString>,
    pub age: ProfileAge,
}

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
    pub profile_verification_status_filter: Option<ProfileVerificationStatusFilter>,
    pub min_distance_km_filter: Option<MinDistanceKm>,
    pub max_distance_km_filter: Option<MaxDistanceKm>,
    pub random_profile_order: bool,
    pub profile_edited_time: ProfileEditedTime,
}

#[derive(Clone, Copy)]
pub struct ProfileModificationMetadata {
    pub version: ProfileVersion,
    pub time: ProfileEditedTime,
}

impl ProfileModificationMetadata {
    pub fn generate() -> Self {
        Self {
            version: ProfileVersion::new_random(),
            time: ProfileEditedTime::current_time(),
        }
    }
}
