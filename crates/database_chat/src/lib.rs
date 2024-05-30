#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod current;
pub mod history;

pub use database::IntoDatabaseError;
pub use model::schema;
