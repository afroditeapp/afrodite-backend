use std::fmt::Debug;

use simple_backend_utils::ComponentError;

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
    #[error("Getting file metadata failed")]
    IoFileMetadata,

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
