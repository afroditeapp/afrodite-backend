//! Account file storage

// TODO: Remove all git related code

use tokio::sync::mpsc;

// TODO: Set max limit for IP
// address changes or something (limit IP address history size)?

pub mod utils;

pub use server_common::data::file::FileError;

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
