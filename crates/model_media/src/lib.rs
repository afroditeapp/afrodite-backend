#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(
    clippy::collapsible_else_if,
    clippy::manual_range_contains,
)]

pub use model::*;

pub mod media;
pub mod media_admin;

pub use media::*;
pub use media_admin::*;

pub mod markers_media;
