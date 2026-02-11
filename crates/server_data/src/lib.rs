#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::while_let_loop, async_fn_in_trait)]

pub use server_common::{
    data::{DataError, IntoDataError},
    result,
};

use self::file::{FileError, utils::FileDir};

pub mod app;
pub mod cache;
pub mod content_processing;
pub mod data_export;
pub mod data_reset;
pub mod db_manager;
pub mod demo;
pub mod event;
pub mod file;
pub mod id;
pub mod index;
pub mod macros;
pub mod profile_attributes;
pub mod read;
pub mod statistics;
pub mod utils;
pub mod write;
pub mod write_commands;
pub mod write_concurrent;

pub use database::{DieselConnection, DieselDatabaseError};
