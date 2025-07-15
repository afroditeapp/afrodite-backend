#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! HTTP API types and request handlers for all servers.

use utoipa::OpenApi;

use self::utils::SecurityApiAccessTokenDefault;

// Routes
pub mod profile;
pub mod profile_admin;
pub mod profile_internal;

pub use server_api::{app, internal_api, utils};
pub use server_common::{data::DataError, result};

// API docs

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        // Profile
        model_profile::GroupValues,
        model_profile::StatisticsProfileVisibility,
        // Profile admin
        model_profile::profile_admin::ProfileStatisticsHistoryValueType,
    )),
    modifiers(&SecurityApiAccessTokenDefault),
)]
pub struct ApiDocProfile;

pub use server_api::db_write;
