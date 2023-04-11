use error_stack::Result;
use serde::de::DeserializeOwned;

use crate::{
    api::model::{AccountId, ApiKey},
    server::database::{ file::utils::AccountDir},
};

use super::{
    FileError, utils::FileDir,
};
use crate::utils::IntoReportExt;


pub struct FileReadCommands<'a> {
    dir: &'a FileDir,
}

impl <'a> FileReadCommands<'a> {
    pub fn new(dir: &'a FileDir) -> Self {
        Self { dir }
    }
}
