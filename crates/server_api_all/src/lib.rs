#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! HTTP API types and request handlers for all servers.

use utoipa::OpenApi;

// Routes
pub mod connection;

pub mod register;

pub use server_api::{app, internal_api, utils};
pub use server_common::{data::DataError, result};

// API docs

#[derive(OpenApi)]
#[openapi(paths(connection::get_connect_websocket,))]
pub struct ApiDocConnection;

pub use server_api::{db_write, db_write_multiple};
