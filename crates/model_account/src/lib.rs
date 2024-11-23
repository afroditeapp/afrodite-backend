#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(
    clippy::collapsible_else_if,
    clippy::manual_range_contains,
)]

pub mod account;
pub mod account_admin;

pub use account::*;
pub use account_admin::*;

mod markers_account;
pub use model::*;
