#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::empty_docs,
    clippy::to_string_trait_impl,
    clippy::too_many_arguments,
)]

#[macro_use]
extern crate serde_derive;

extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate url;

#[rustfmt::skip]
pub mod apis;

#[rustfmt::skip]
pub mod models;


pub mod manual_additions;
