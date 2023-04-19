#![allow(clippy::derive_partial_eq_without_eq)]

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
