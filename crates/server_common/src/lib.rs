#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

// TODO(prod): Remove webpki-roots dependency

pub mod app;
pub mod data;
pub mod internal_api;
pub mod push_notifications;
pub mod result;
pub mod websocket;
