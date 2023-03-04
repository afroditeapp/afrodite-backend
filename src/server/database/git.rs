pub mod file;
pub mod read;
pub mod util;
pub mod write;

use std::path::Path;

use crate::utils::IntoReportExt;
use error_stack::Result;
use git2::{Repository, Signature, Tree};
use tokio::sync::mpsc;

use {file::GetGitPath, util::GitUserDirPath};

const REPOSITORY_USER_NAME: &str = "Pihka backend";
const REPOSITORY_USER_EMAIL: &str = "email";
const INITIAL_COMMIT_MSG: &str = "Initial commit";

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
    repository: Repository,
    profile: &'a GitUserDirPath,
}

impl<'a> GitDatabase<'a> {
    /// Create git repository and store user id there
    pub fn create(profile: &'a GitUserDirPath) -> Result<Self, GitError> {
        let repository = Repository::init(profile.path()).into_error(GitError::Init)?;

        let mut repository = Self {
            repository,
            profile,
        };

        repository.initial_commit()?;

        Ok(repository)
    }

    pub fn open(profile: &'a GitUserDirPath) -> Result<Self, GitError> {
        let repository = Repository::open(profile.path()).into_error(GitError::Open)?;

        Ok(Self {
            repository,
            profile,
        })
    }

    pub fn commit<T: GetGitPath>(&mut self, file: T, message: &str) -> Result<(), GitError> {
        let signature = Self::default_signature()?;

        let tree = self.write_to_index(Some(file.git_path().as_str()))?;

        let current_head = self.repository.head().into_error(GitError::Head)?;
        let parent = self
            .repository
            .find_commit(
                current_head
                    .target()
                    .ok_or(GitError::HeadDoesNotPointToCommit)?,
            )
            .into_error(GitError::FindCommit)?;

        self.repository
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[&parent],
            )
            .into_error(GitError::Commit)?;

        Ok(())
    }

    // File path is relative to git repository root.
    fn initial_commit(&mut self) -> Result<(), GitError> {
        let signature = Self::default_signature()?;

        let tree = self.write_to_index::<&str>(None)?;

        self.repository
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                INITIAL_COMMIT_MSG,
                &tree,
                &[],
            )
            .into_error(GitError::Commit)?;

        Ok(())
    }

    // File path is relative to git repository root.
    fn write_to_index<T: AsRef<Path>>(&self, file: Option<T>) -> Result<Tree<'_>, GitError> {
        let tree_id = {
            let mut index = self.repository.index().into_error(GitError::Index)?;
            if let Some(file) = file {
                index
                    .add_path(file.as_ref())
                    .into_error(GitError::AddPath)?;
            }
            index.write_tree().into_error(GitError::WriteTree)?
        };
        self.repository
            .find_tree(tree_id)
            .into_error(GitError::FindTree)
    }

    fn default_signature() -> Result<Signature<'static>, GitError> {
        Signature::now(REPOSITORY_USER_NAME, REPOSITORY_USER_EMAIL)
            .into_error(GitError::SignatureCreation)
    }
}
