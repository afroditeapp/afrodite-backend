use error_stack::Result;
use serde::Serialize;

use std::io::Write;
use tracing::error;

use super::file::{};
use super::utils::FileDir;
use super::{FileError};

use crate::utils::IntoReportExt;
use crate::{
    api::model::ApiKey,
    server::database::{
        file::file::{},
        file::utils::AccountDir,
        FileOperationHandle,
    },
};

pub struct FileWriteCommands<'a> {
    dir: &'a FileDir,
}

impl <'a> FileWriteCommands<'a> {
    pub fn new(
        dir: &'a FileDir,
    ) -> Self {
        Self { dir }
    }
}
