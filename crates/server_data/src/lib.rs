#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

#![allow(clippy::while_let_loop)]



use self::{
    cache::{CacheError, DatabaseCache},
    file::{utils::FileDir, FileError},
};


pub use server_common::{
    data::{DataError, IntoDataError},
    result,
};

pub mod app;
pub mod cache;
pub mod content_processing;
pub mod event;
pub mod file;
pub mod index;
pub mod read;
pub mod utils;
pub mod write;
pub mod write_commands;
pub mod write_concurrent;
pub mod db_manager;
pub mod macros;

// TODO: Remove?
pub type DatabeseEntryId = String;


pub use database::{
    DieselDatabaseError,
    DieselConnection,
    current::read::CurrentSyncReadCommands,
    current::write::CurrentSyncWriteCommands,
};
