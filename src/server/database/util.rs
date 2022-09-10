use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::api::core::user::UserId;

use super::{
    command::{read::DatabaseReadCommands, write::DatabaseWriteCommands},
    file::GitRepositoryPath,
    DatabaseOperationHandle,
};

/// Path to directory which contains all profile directories.
///
/// One profile directory contains one git repository.
#[derive(Debug, Clone)]
pub struct DatabasePath {
    database_dir: PathBuf,
}

impl DatabasePath {
    pub fn new<T: ToOwned<Owned = PathBuf>>(database_dir: T) -> Self {
        Self {
            database_dir: database_dir.to_owned(),
        }
    }

    /// Make sure that `id` does not contain special characters
    pub fn profile_dir(&self, id: &str) -> ProfileDirPath {
        ProfileDirPath {
            git_repository_path: self.database_dir.join(id),
            id: id.to_owned(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.database_dir
    }
}

// Directory to profile directory which contains git repository.
#[derive(Debug, Clone)]
pub struct ProfileDirPath {
    git_repository_path: PathBuf,
    id: UserId,
}

impl ProfileDirPath {
    pub fn path(&self) -> &PathBuf {
        &self.git_repository_path
    }

    pub fn exists(&self) -> bool {
        self.exists()
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    /// Use this only if you know that file does not exist or it is not opened
    /// for reading.
    pub fn create_file<T: GitRepositoryPath>(&self, file: T) -> io::Result<fs::File> {
        fs::File::create(self.git_repository_path.join(file.relative_path()))
    }

    /// Open file for reading.
    pub fn open_file<T: GitRepositoryPath>(&self, file: T) -> io::Result<fs::File> {
        fs::File::open(self.git_repository_path.join(file.relative_path()))
    }

    /// Replace file using new file.
    pub fn replace_file<T: GitRepositoryPath, U: FnMut(&mut fs::File) -> Result<(), io::Error>>(
        &self,
        file: T,
        mut write_handle: U,
        commit_msg: &str,
    ) -> io::Result<()> {
        // Check that previous replace was successfull

        let file = self.git_repository_path.join(file.relative_path());
        let tmp_file = file.join(".tmp");
        let mut tmp = fs::File::create(&tmp_file)?;
        write_handle(&mut tmp)?;
        fs::rename(&tmp_file, file)
        // TODO: error handling?
    }
}

pub struct WriteGuard {
    profile: ProfileDirPath,
    database_handle: DatabaseOperationHandle,
}

impl WriteGuard {
    pub fn new(profile: ProfileDirPath, database_handle: DatabaseOperationHandle) -> Self {
        Self {
            profile,
            database_handle,
        }
    }

    pub fn read(&self) -> DatabaseReadCommands<'_> {
        DatabaseReadCommands::new(&self.profile)
    }

    pub fn write(&mut self) -> DatabaseWriteCommands {
        DatabaseWriteCommands::new(self.profile.clone(), self.database_handle.clone())
    }
}
