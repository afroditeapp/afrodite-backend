//! Account file storage

// TODO: Remove all git related code

use tokio::sync::mpsc;

use ::utils::ComponentError;

pub mod file;
pub mod read;
pub mod utils;
pub mod write;

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
