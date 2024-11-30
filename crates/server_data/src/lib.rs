#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::while_let_loop, async_fn_in_trait)]

pub use server_common::{
    data::{DataError, IntoDataError},
    result,
};

use self::file::{utils::FileDir, FileError};

pub mod app;
pub mod id;
pub mod cache;
pub mod content_processing;
pub mod db_manager;
pub mod event;
pub mod file;
pub mod index;
pub mod macros;
pub mod read;
pub mod utils;
pub mod write;
pub mod write_commands;
pub mod write_concurrent;
pub mod demo;
pub mod statistics;

// TODO: Remove?
pub type DatabeseEntryId = String;

pub use database::{
    DieselConnection, DieselDatabaseError,
};
