#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

#![allow(
    async_fn_in_trait,
    clippy::single_match,
    clippy::while_let_loop,
)]

pub mod api;
pub mod client;
pub mod server;
pub mod utils;
