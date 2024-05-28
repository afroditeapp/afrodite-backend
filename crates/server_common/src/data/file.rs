
use std::{
    fmt::Debug,
    fs,
    path::Path,
    sync::Arc,
};

use config::Config;
use database::{
    CurrentReadHandle, CurrentWriteHandle, ErrorContext, HistoryReadHandle, HistoryWriteHandle,
};
use error_stack::Context;
use model::{AccountId, AccountIdInternal, EmailAddress, IsLoggingAllowed, SignInWithInfo};
use simple_backend::media_backup::MediaBackupHandle;
use simple_backend_database::{DatabaseHandleCreator, DbReadCloseHandle, DbWriteCloseHandle};
use simple_backend_utils::ComponentError;
use tracing::info;

use crate::{result::WrappedReport};

impl ComponentError for FileError {
    const COMPONENT_NAME: &'static str = "File";
}

#[derive(thiserror::Error, Debug)]
pub enum FileError {
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
    #[error("File flushing failed")]
    IoFileFlush,
    #[error("File sync failed")]
    IoFileSync,
    #[error("File remove failed")]
    IoFileRemove,
    #[error("Iterating directory contents failed")]
    IoDirIter,

    #[error("Missing file name")]
    MissingFileName,
    #[error("Invalid file name")]
    InvalidFileName,
    #[error("Invalid directory")]
    InvalidDirectory,

    // Serde
    #[error("Serde serialization failed")]
    SerdeSerialize,
    #[error("Serde deserialization failed")]
    SerdeDerialize,

    #[error("AccountId parsing error")]
    AccountIdParsing,

    #[error("Stream reading failed")]
    StreamReadFailed,
}
