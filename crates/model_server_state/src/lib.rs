#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::collapsible_else_if, clippy::manual_range_contains)]

//! Data types for API and database.

pub use model::{Db, schema};

mod account;
pub use account::*;

mod markers_server_state;
