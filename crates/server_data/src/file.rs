//! Account file storage

// TODO(refactor): Remove all git related code. (DataError)

use tokio::sync::mpsc;

pub mod utils;

pub use server_common::data::file::FileError;

use crate::db_manager::{InternalReading, InternalWriting};

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

pub trait FileRead {
    fn files(&self) -> &crate::FileDir;
}

impl<I: InternalReading> FileRead for I {
    fn files(&self) -> &crate::FileDir {
        self.root().file_dir()
    }
}

pub trait FileWrite {
    fn files(&self) -> &crate::FileDir;
}

impl<I: InternalWriting> FileWrite for I {
    fn files(&self) -> &crate::FileDir {
        self.root().file_dir()
    }
}
