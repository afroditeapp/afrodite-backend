#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::collapsible_else_if, clippy::manual_range_contains)]

//! Data types for API and database.

pub use simple_backend_model::UnixTime;

pub mod common;
pub use common::*;

pub mod common_admin;
pub use common_admin::*;

pub mod common_history;
pub use common_history::*;

pub mod account;
pub use account::*;

pub mod chat;
pub use chat::*;

pub mod media;
pub use media::*;

pub mod profile;
pub use profile::*;

pub mod db_only;
pub use db_only::*;

pub mod markers;
pub mod schema;
pub mod schema_sqlite_types;

pub type Db = diesel::sqlite::Sqlite;

#[derive(thiserror::Error, Debug)]
pub enum EnumParsingError {
    #[error("ParsingFailed, value: {0}")]
    ParsingError(i64),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NextNumberStorage {
    next: i64,
}

impl NextNumberStorage {
    pub fn get_and_increment(&mut self) -> i64 {
        let next = self.next;
        self.next = self.next.wrapping_add(1);
        next
    }
}

#[derive(utoipa::ToSchema)]
#[schema(value_type = String, format = Binary)]
pub struct BinaryData(());
