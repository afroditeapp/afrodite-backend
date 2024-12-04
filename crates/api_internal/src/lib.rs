#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! This crate provides a wrapper for the internal API of the server.
//! Prevents exposing api_client crate model types to server code.

pub use api_client::apis::{configuration::Configuration, Error};

/// Wrapper for server internal API with correct model types.
pub struct InternalApi;
