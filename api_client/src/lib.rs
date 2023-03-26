#![allow(clippy::derive_partial_eq_without_eq)]


#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate url;
extern crate reqwest;

pub mod apis;
pub mod models;


pub mod manual_additions;
