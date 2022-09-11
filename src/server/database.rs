pub mod command;
pub mod file;
pub mod git;
pub mod util;

use std::{
    io
};

use tokio::{
    sync::{mpsc},
};

use self::{
    git::{GitError},
};

pub type DatabeseEntryId = String;

/// Every running database write operation should keep this handle. When server
/// quit is started main function waits that all handles are dropped.
#[derive(Debug, Clone)]
pub struct DatabaseOperationHandle {
    _sender: mpsc::Sender<()>,
}

impl DatabaseOperationHandle {
    pub fn new() -> (Self, mpsc::Receiver<()>) {
        let (_sender, receiver) = mpsc::channel(1);
        (Self { _sender }, receiver)
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    Git(GitError),
    FileCreate(io::Error),
    FileOpen(io::Error),
    FileRename(io::Error),
    FileIo(io::Error),
    Serialize(serde_json::Error),
}
