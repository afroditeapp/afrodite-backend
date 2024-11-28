#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(
    clippy::collapsible_else_if,
    clippy::manual_range_contains,
)]

//! Data types for API and database.

pub use model::{schema, schema_sqlite_types, Db};

mod account;
pub use account::*;

mod chat;
pub use chat::*;

mod media;
pub use media::*;

mod profile;
pub use profile::*;

mod markers_server_data;
