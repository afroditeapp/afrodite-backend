#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(
    clippy::collapsible_else_if,
    clippy::manual_range_contains,
)]

//! Data types for API and database.

pub use simple_backend_model::UnixTime;

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
pub use profile_admin::*;

pub type Db = diesel::sqlite::Sqlite;


#[derive(thiserror::Error, Debug)]
pub enum EnumParsingError {
    #[error("ParsingFailed, value: {0}")]
    ParsingError(i64),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NextNumberStorage {
    next: i64
}

impl NextNumberStorage {
    fn get_and_increment(&mut self) -> i64 {
        let next = self.next;
        self.next = self.next.wrapping_add(1);
        next
    }
}

#[derive(utoipa::ToSchema)]
#[schema(value_type = String, format = Binary)]
pub struct BinaryData(());
