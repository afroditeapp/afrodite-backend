#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod current;
pub mod history;

pub use model::schema;
pub use database::IntoDatabaseError;

use model::FcmDeviceToken;
