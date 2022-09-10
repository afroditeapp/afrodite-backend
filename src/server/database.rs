pub mod git;
pub mod command;
pub mod file;
pub mod util;

use std::{sync::Arc, path::{Path, PathBuf}, io, fs};

use tokio::{sync::{oneshot, mpsc}, task::{JoinHandle, spawn_blocking}};

use crate::{config::Config, utils::{QuitSender, QuitReceiver}, api::core::user::{LoginBody, LoginResponse, UserId, UserApiToken}};

use self::{git::{GitDatabase, GitError}, file::{GitRepositoryPath, CoreFile}, util::{DatabasePath, ProfileDirPath}};

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
