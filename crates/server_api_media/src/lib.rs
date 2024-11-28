#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! HTTP API types and request handlers for all servers.

use utoipa::OpenApi;

use self::utils::SecurityApiAccessTokenDefault;

// Routes
pub mod media;
pub mod media_admin;
pub mod media_internal;

pub use server_api::{app, utils};
pub use server_common::{data::DataError, result};

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        // Media internal
        media_internal::internal_get_check_moderation_request_for_account,
    ),
    components(schemas(
        // Media admin
        model_media::ModerationQueueType,
    )),
    modifiers(&SecurityApiAccessTokenDefault),
)]
pub struct ApiDocMedia;

pub use server_api::{db_write, db_write_multiple};
