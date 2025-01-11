#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

#![allow(
    clippy::single_match,
    clippy::while_let_loop,
)]

pub mod api;
pub mod client;
pub mod config;
pub mod server;
pub mod utils;
