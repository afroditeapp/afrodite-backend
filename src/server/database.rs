pub mod command;
pub mod file;
pub mod git;
pub mod util;

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{
    sync::{mpsc, oneshot},
    task::{spawn_blocking, JoinHandle},
};

use crate::{
    api::core::user::{LoginBody, LoginResponse, UserApiToken, UserId},
    config::Config,
    utils::{QuitReceiver, QuitSender},
};

use self::{
    file::{CoreFile, GitRepositoryPath},
    git::{GitDatabase, GitError},
    util::{DatabasePath, ProfileDirPath},
};

use crate::api::core::user::{RegisterBody, RegisterResponse};

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
}
