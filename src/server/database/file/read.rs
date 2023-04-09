use error_stack::Result;
use serde::de::DeserializeOwned;

use crate::{
    api::model::{AccountId, ApiKey},
    server::database::{ file::utils::AccountFilesDir},
};

use super::{
    GitError,
};
use crate::utils::IntoReportExt;

/// Reading can be done async as Git library is not used.
pub struct GitDatabaseReadCommands {
    account_dir: AccountFilesDir,
}

impl<'a> GitDatabaseReadCommands {
    pub fn new(account_dir: AccountFilesDir) -> Self {
        Self { account_dir }
    }

}
