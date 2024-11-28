#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(
    clippy::collapsible_else_if,
    clippy::manual_range_contains,
)]

pub use model::*;
pub use model_server_data::*;

pub mod profile;
pub mod profile_admin;

pub use profile::*;
pub use profile_admin::*;
