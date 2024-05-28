#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::collapsible_else_if)]

//! Data types for API and database.

pub mod account;
pub mod account_admin;
pub mod chat;
pub mod chat_admin;
pub mod common;
pub mod common_admin;
pub mod media;
pub mod media_admin;
pub mod profile;
pub mod profile_admin;

mod markers;
pub mod schema;
mod schema_sqlite_types;

pub use account::*;
pub use account_admin::*;
pub use chat::*;
pub use common::*;
pub use common_admin::*;
pub use markers::*;
pub use media::*;
pub use media_admin::*;
pub use profile::*;

pub type Db = diesel::sqlite::Sqlite;
