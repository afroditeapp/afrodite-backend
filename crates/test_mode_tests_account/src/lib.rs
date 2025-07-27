#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! Account API tests

mod admin;
mod initial_setup;

pub fn call_this_to_make_sure_that_crate_is_linked() {}
