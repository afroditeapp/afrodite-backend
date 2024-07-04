#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! HTTP API types and request handlers for all servers.

use utoipa::OpenApi;

use self::utils::SecurityApiAccessTokenDefault;

// Routes
pub mod profile;
pub mod profile_internal;

pub use server_api::{app, internal_api, utils};
pub use server_common::{data::DataError, result};

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        // Profile
        profile::get_profile,
        profile::get_profile_from_database_debug_mode_benchmark,
        profile::get_location,
        profile::get_favorite_profiles,
        profile::get_available_profile_attributes,
        profile::get_search_groups,
        profile::get_search_age_range,
        profile::get_profile_attribute_filters,
        profile::post_get_next_profile_page,
        profile::post_profile,
        profile::post_profile_to_database_debug_mode_benchmark,
        profile::post_reset_profile_paging,
        profile::post_favorite_profile,
        profile::post_search_groups,
        profile::post_search_age_range,
        profile::post_profile_attribute_filters,
        profile::put_location,
        profile::delete_favorite_profile,
    ),
    components(schemas(
        // Profile
        model::profile::Profile,
        model::profile::ProfilePage,
        model::profile::ProfileLink,
        model::profile::ProfileVersion,
        model::profile::ProfileUpdate,
        model::profile::ProfileAge,
        model::profile::ProfileSearchAgeRange,
        model::profile::GetProfileResult,
        model::profile::GetProfileQueryParam,
        model::profile::Location,
        model::profile::FavoriteProfilesPage,
        model::profile::AvailableProfileAttributes,
        model::profile::ProfileAttributes,
        model::profile::ProfileAttributesSyncVersion,
        model::profile::ProfileAttributeValue,
        model::profile::ProfileAttributeValueUpdate,
        model::profile::ProfileAttributeFilterValue,
        model::profile::ProfileAttributeFilterValueUpdate,
        model::profile::ProfileAttributeFilterList,
        model::profile::ProfileAttributeFilterListUpdate,
        model::profile::Attribute,
        model::profile::AttributeOrderMode,
        model::profile::AttributeMode,
        model::profile::AttributeValue,
        model::profile::AttributeValueOrderMode,
        model::profile::Language,
        model::profile::Translation,
        model::profile::GroupValues,
        model::profile::IconResource,
        model::profile::IconLocation,
        model::profile::SearchGroups,
        model::profile::LastSeenTime,
    )),
    modifiers(&SecurityApiAccessTokenDefault),
)]
pub struct ApiDocProfile;

pub use server_api::{db_write, db_write_multiple};
