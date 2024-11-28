#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! HTTP API types and request handlers for all servers.

use utoipa::OpenApi;

use self::utils::SecurityApiAccessTokenDefault;

// Routes
pub mod common;
pub mod common_admin;
pub mod common_internal;

pub mod utils;

pub use server_common::{data::DataError, result};
pub use server_state::{
    internal_api,
    create_open_api_router,
    db_write,
    db_write_multiple,
    db_write_raw,
    S
};

pub mod app {
    pub use server_state::app::*;
}

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        // Common
        common::get_version,
        common::get_connect_websocket,
    ),
    components(schemas(
        // Common
        model::common::EventToClient,
        // Manager
        manager_model::SoftwareOptions,
    )),
    modifiers(&SecurityApiAccessTokenDefault),
)]
pub struct ApiDocCommon;
