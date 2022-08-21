use std::{path::{PathBuf, Path}, io::Write};

use git2::{Repository, Signature};

const REPOSITORY_USER_NAME: &str = "Pihka backend";
const REPOSITORY_USER_EMAIL: &str = "email";
const ID_FILE_NAME: &str = "id";
const INITIAL_COMMIT_MSG: &str = "Initial commit";

#[derive(Debug)]
pub enum GitDatabaseError {
    Git2(git2::Error),
    HeadDoesNotPointToCommit,
    CreateFailedAlreadyExists,
    CreateFailedIdFileFailed(std::io::Error),
    OpenFailedDoesNotExists,

}

impl From<git2::Error> for GitDatabaseError {
    fn from(e: git2::Error) -> Self {
        Self::Git2(e)
    }
}

pub struct GitDatabase {
    //path: PathBuf,
    repository: Repository,
}

impl GitDatabase {
    pub fn create(path: &Path, id: &str) -> Result<Self, GitDatabaseError> {
        if path.exists() {
            Err(GitDatabaseError::CreateFailedAlreadyExists)
        } else {
            let repository = Repository::init(&path)?;

            // let mut config = repository.config()?;
            // config.set_str("user.name", REPOSITORY_USER_NAME)?;
            // config.set_str("user.email", REPOSITORY_USER_EMAIL)?;

            let mut repository = Self {
                //path,
                repository,
            };

            let mut id_file = path.to_owned();
            id_file.push(ID_FILE_NAME);
            let mut file = std::fs::File::create(&id_file).map_err(GitDatabaseError::CreateFailedIdFileFailed)?;
            file.write_all(id.as_bytes()).map_err(GitDatabaseError::CreateFailedIdFileFailed)?;
            drop(file); // Make sure that file is closed, so it is included in the commit.

            repository.initial_commit(Path::new(ID_FILE_NAME))?;

            Ok(repository)
        }
    }

    pub fn open(path: &Path) -> Result<Self, GitDatabaseError> {
        if path.exists() {
            let repository = Repository::open(&path)?;

            Ok(Self {
                //path,
                repository,
            })
        } else {
            Err(GitDatabaseError::OpenFailedDoesNotExists)
        }
    }

    pub fn initial_commit(&mut self, file: &Path) -> Result<(), GitDatabaseError> {
        let signature = Self::default_signature()?;

        let tree_id = {
            let mut index = self.repository.index()?;
            index.add_path(file)?;
            index.write_tree()?
        };
        let tree = self.repository.find_tree(tree_id)?;

        self.repository.commit(Some("HEAD"), &signature, &signature, INITIAL_COMMIT_MSG, &tree, &[])?;

        Ok(())
    }

    pub fn commit(&mut self, file: &Path, message: &str) -> Result<(), GitDatabaseError> {
        let signature = Self::default_signature()?;

        let tree_id = {
            let mut index = self.repository.index()?;
            index.add_path(file)?;
            index.write_tree()?
        };
        let tree = self.repository.find_tree(tree_id)?;
        let current_head = self.repository.head()?;
        let parent = self.repository.find_commit(
            current_head.target().ok_or(GitDatabaseError::HeadDoesNotPointToCommit)?
        )?;

        self.repository.commit(Some("HEAD"), &signature, &signature, message, &tree, &[&parent])?;

        Ok(())
    }

    fn default_signature() -> Result<Signature<'static>, git2::Error> {
        Signature::now(REPOSITORY_USER_NAME, REPOSITORY_USER_EMAIL)
    }
}
