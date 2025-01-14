#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

// TODO: Fix lettre to not include unused webpki-roots dependency
//       in dependency tree when rustls and native certificates are enabled.

pub mod app;
pub mod data;
pub mod internal_api;
pub mod push_notifications;
pub mod result;
pub mod websocket;
