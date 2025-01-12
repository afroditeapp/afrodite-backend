#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

// TODO(prod): Remove webpki-roots dependency

// Ignore warning about unused yup_oauth2.
// This library depends on yup_oauth2 because
// fcm library does not have feature flag for
// using reqwest with rustls and native roots.
use yup_oauth2 as _;

pub mod app;
pub mod data;
pub mod internal_api;
pub mod push_notifications;
pub mod result;
pub mod websocket;
