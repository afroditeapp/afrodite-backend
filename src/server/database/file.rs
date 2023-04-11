//! Account file storage

// TODO: Remove all git related code

pub mod file;
pub mod read;
pub mod utils;
pub mod write;

use error_stack::Result;
use tokio::sync::mpsc;

use utils::AccountDir;

/// Every running database write operation should keep this handle. When server
/// quit is started main function waits that all handles are dropped.
#[derive(Debug, Clone)]
pub struct FileOperationHandle {
    _sender: mpsc::Sender<()>,
}

impl FileOperationHandle {
    pub fn new() -> (Self, mpsc::Receiver<()>) {
        let (_sender, receiver) = mpsc::channel(1);
        (Self { _sender }, receiver)
    }
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

    // Serde
    #[error("Serde serialization failed")]
    SerdeSerialize,
    #[error("Serde deserialization failed")]
    SerdeDerialize,

    #[error("AccountId parsing error")]
    AccountIdParsing,
}
