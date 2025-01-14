#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! HTTP API types and request handlers for all servers.

use utoipa::OpenApi;

use self::utils::SecurityApiAccessTokenDefault;

// Routes
pub mod account;
pub mod account_admin;
pub mod account_bot;
pub mod app;

pub use server_api::{internal_api, utils};
pub use server_common::{data::DataError, result};

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        // Account
        account::post_sign_in_with_login,
        // Account bot API
        account_bot::post_register,
        account_bot::post_login,
    ),
    modifiers(&SecurityApiAccessTokenDefault),
)]
pub struct ApiDocAccount;

pub use server_api::{db_write, db_write_multiple};
