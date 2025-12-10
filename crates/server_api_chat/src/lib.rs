#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! HTTP API types and request handlers for all servers.

use utoipa::OpenApi;

use self::utils::SecurityApiAccessTokenDefault;

// Routes
pub mod chat;
pub mod chat_admin;

pub use server_api::{app, utils};
pub use server_common::{data::DataError, result};

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        chat::transfer::get_backup_transfer,
    ),
    components(schemas(
        model_chat::ClientRole,
        model_chat::BackupTransferInitialMessage,
        model_chat::BackupTransferData,
        model_chat::BackupTransferByteCount,
    )),
    modifiers(&SecurityApiAccessTokenDefault),
)]
pub struct ApiDocChat;

pub use server_api::db_write;
