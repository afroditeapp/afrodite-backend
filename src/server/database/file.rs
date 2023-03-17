//! Account file storage

// TODO: Remove all git related code

pub mod file;
pub mod read;
pub mod utils;
pub mod write;

use std::path::Path;

use crate::utils::IntoReportExt;
use error_stack::Result;
use tokio::sync::mpsc;

use {file::GetGitPath, utils::GitUserDirPath};


/// Every running database write operation should keep this handle. When server
/// quit is started main function waits that all handles are dropped.
#[derive(Debug, Clone)]
pub struct GitDatabaseOperationHandle {
    _sender: mpsc::Sender<()>,
}

impl GitDatabaseOperationHandle {
    pub fn new() -> (Self, mpsc::Receiver<()>) {
        let (_sender, receiver) = mpsc::channel(1);
        (Self { _sender }, receiver)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GitError {
    #[error("Initializing Git repository failed")]
    Init,
    #[error("Creating Git commit signature failed")]
    SignatureCreation,
    #[error("Opening Git repository failed")]
    Open,
    #[error("Git repository head does not point to a commit")]
    HeadDoesNotPointToCommit,
    #[error("Get index file failed")]
    Index,
    #[error("Adding file to index failed")]
    AddPath,
    #[error("Writing tree failed")]
    WriteTree,
    #[error("Finding tree failed")]
    FindTree,
    #[error("Getting repository HEAD failed")]
    Head,
    #[error("Finding commit failed")]
    FindCommit,
    #[error("Creating commit failed")]
    Commit,

    // File IO errors
    #[error("File create failed")]
    IoFileCreate,
    #[error("File open failed")]
    IoFileOpen,
    #[error("File rename failed")]
    IoFileRename,
    #[error("File reading failed")]
    IoFileRead,
    #[error("File writing failed")]
    IoFileWrite,

    // Serde
    #[error("Serde serialization failed")]
    SerdeSerialize,
    #[error("Serde deserialization failed")]
    SerdeDerialize,

    #[error("AccountId parsing error")]
    AccountIdParsing,
}

/// Git database for one user
pub struct GitDatabase<'a> {
    profile: &'a GitUserDirPath,
}

impl<'a> GitDatabase<'a> {
    pub fn create(profile: &'a GitUserDirPath) -> Result<Self, GitError> {

        let mut repository = Self {
            profile,
        };

        Ok(repository)
    }

    pub fn open(profile: &'a GitUserDirPath) -> Result<Self, GitError> {

        Ok(Self {
            profile,
        })
    }

}
