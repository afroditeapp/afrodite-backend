pub mod read;
pub mod write;
pub mod file;
pub mod util;

use std::{
    io::{Write, self},
    path::{Path, PathBuf}, fs,
};

use git2::{Repository, Signature, Tree};
use tokio::sync::mpsc;

use {
    file::{CoreFile, GetGitPath},
    util::GitUserDirPath,
};

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

#[derive(Debug)]
pub enum GitError {
    Init(git2::Error),
    SignatureCreation(git2::Error),
    Open(git2::Error),
    HeadDoesNotPointToCommit,
    CreateIdFile(std::io::Error),
    Index(git2::Error),
    AddFile(git2::Error),
    AddPath(git2::Error),
    WriteTree(git2::Error),
    FindTree(git2::Error),
    Head(git2::Error),
    FindCommit(git2::Error),
    Commit(git2::Error),
}

/// Git database for one user
pub struct GitDatabase<'a> {
    repository: Repository,
    profile: &'a GitUserDirPath,
}

impl<'a> GitDatabase<'a> {
    /// Create git repository and store user id there
    pub fn create(profile: &'a GitUserDirPath) -> Result<Self, GitError> {
        let repository = Repository::init(profile.path()).map_err(GitError::Init)?;

        let mut repository = Self {
            repository,
            profile,
        };

        let mut file = repository.create_raw_file(CoreFile::Id)
            .map_err(GitError::CreateIdFile)?;
        file.write_all(profile.id().as_str().as_bytes())
            .map_err(GitError::CreateIdFile)?;
        drop(file); // Make sure that file is closed, so it is included in the commit.

        repository.initial_commit(CoreFile::Id.git_path().as_str())?;

        Ok(repository)
    }

    pub fn open(profile: &'a GitUserDirPath) -> Result<Self, GitError> {
        let repository = Repository::open(profile.path()).map_err(GitError::Open)?;

        Ok(Self {
            repository,
            profile,
        })
    }

    pub fn commit<T: GetGitPath>(&mut self, file: T, message: &str) -> Result<(), GitError> {
        let signature = Self::default_signature()?;

        let tree = self.write_to_index(file.git_path().as_str())?;

        let current_head = self.repository.head().map_err(GitError::Head)?;
        let parent = self
            .repository
            .find_commit(
                current_head
                    .target()
                    .ok_or(GitError::HeadDoesNotPointToCommit)?,
            )
            .map_err(GitError::FindCommit)?;

        self.repository
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[&parent],
            )
            .map_err(GitError::Commit)?;

        Ok(())
    }

    // File path is relative to git repository root.
    fn initial_commit<T: AsRef<Path>>(&mut self, file: T) -> Result<(), GitError> {
        let signature = Self::default_signature()?;

        let tree = self.write_to_index(file.as_ref())?;

        self.repository
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                INITIAL_COMMIT_MSG,
                &tree,
                &[],
            )
            .map_err(GitError::Commit)?;

        Ok(())
    }

    // File path is relative to git repository root.
    fn write_to_index<T: AsRef<Path>>(&self, file: T) -> Result<Tree<'_>, GitError> {
        let tree_id = {
            let mut index = self.repository.index().map_err(GitError::Index)?;
            index.add_path(file.as_ref()).map_err(GitError::AddPath)?;
            index.write_tree().map_err(GitError::WriteTree)?
        };
        self.repository
            .find_tree(tree_id)
            .map_err(GitError::FindTree)
    }

    fn default_signature() -> Result<Signature<'static>, GitError> {
        Signature::now(REPOSITORY_USER_NAME, REPOSITORY_USER_EMAIL)
            .map_err(GitError::SignatureCreation)
    }

    /// Create new file which should be committed to Git.
    pub fn create_raw_file<T: GetGitPath>(&self, file: T) -> Result<fs::File, io::Error> {
        let path = self.profile.path().join(file.git_path().as_str());
        fs::File::create(path)
    }
}
