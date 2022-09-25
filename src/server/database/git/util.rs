use std::{
    fs,
    path::{Path, PathBuf}, future::Future,
};


use crate::api::core::user::UserId;

use super::{
    {read::GitDatabaseReadCommands},
    file::{GetGitPath, GetLiveVersionPath, GetTmpPath},
    GitDatabase, super::DatabaseError,
};

/// Path to directory which contains all user data git directories.
///
/// One user directory contains one git repository.
#[derive(Debug, Clone)]
pub struct DatabasePath {
    database_dir: PathBuf,
}

impl DatabasePath {
    pub fn new<T: AsRef<Path>>(database_dir: T) -> Self {
        Self {
            database_dir: database_dir.as_ref().to_path_buf(),
        }
    }

    /// Make sure that `id` does not contain special characters
    pub fn user_git_dir(&self, id: &UserId) -> GitUserDirPath {
        GitUserDirPath {
            git_repository_path: self.database_dir.join(id.as_str()),
            id: id.clone(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.database_dir
    }

    // pub async fn iter_users<
    //     T: FnMut(GitUserDirPath) -> S,
    //     S: Future<Output = Result<(), DatabaseError>>,
    // >(&self, mut handle_user_dir: T) -> Result<(), DatabaseError> {
    //     let mut user_dirs = tokio::fs::read_dir(&self.database_dir).await?;

    //     while let Some(dir_entry) = user_dirs.next_entry().await? {
    //         let user_id_string = dir_entry.file_name().into_string().map_err(|_| DatabaseError::Utf8)?;
    //         let user_dir = self.user_git_dir(&UserId::new(user_id_string));

    //         handle_user_dir(user_dir).await?;
    //     }

    //     Ok(())
    // }
}

// Directory to profile directory which contains git repository.
#[derive(Debug, Clone)]
pub struct GitUserDirPath {
    /// Absolute path to profile directory.
    git_repository_path: PathBuf,
    /// User id which is also directory name.
    id: UserId,
}

impl GitUserDirPath {
    /// Absolute path to profile directory
    pub fn path(&self) -> &PathBuf {
        &self.git_repository_path
    }

    pub fn exists(&self) -> bool {
        self.git_repository_path.exists()
    }

    pub fn id(&self) -> &UserId {
        &self.id
    }

    pub async fn read_to_string_optional<T: GetLiveVersionPath>(&self, file: T) -> Result<Option<String>, DatabaseError> {
        let path = self.git_repository_path.join(file.live_path().as_str());
        if !path.is_file() {
            return Ok(None);
        }
        tokio::fs::read_to_string(path).await.map_err(DatabaseError::FileIo).map(Some)
    }

    pub async fn read_to_string<T: GetLiveVersionPath>(&self, file: T) -> Result<String, DatabaseError> {
        let path = self.git_repository_path.join(file.live_path().as_str());
        tokio::fs::read_to_string(path).await.map_err(DatabaseError::FileIo)
    }

    /// Open file for reading.
    pub fn open_file<T: GetLiveVersionPath>(&self, file: T) -> Result<fs::File, DatabaseError> {
        let path = self.git_repository_path.join(file.live_path().as_str());
        fs::File::open(path).map_err(DatabaseError::FileOpen)
    }

    /// Replace file using new file.
    pub fn replace_file<
        T: GetGitPath + GetLiveVersionPath + Copy,
        U: FnMut(&mut fs::File) -> Result<(), DatabaseError>,
    >(
        &self,
        file: T,
        commit_msg: &str,
        mut write_handle: U,
    ) -> Result<(), DatabaseError> {
        let git_file_path = self.git_repository_path.join(file.git_path().as_str());
        let mut git_file = fs::File::create(&git_file_path).map_err(DatabaseError::FileCreate)?;

        write_handle(&mut git_file)?;
        drop(git_file);

        let mut git = GitDatabase::open(self).map_err(DatabaseError::Git)?;
        git.commit(file, commit_msg).map_err(DatabaseError::Git)?;

        let live_file_path = self.git_repository_path.join(file.live_path().as_str());
        fs::rename(&git_file_path, live_file_path).map_err(DatabaseError::FileRename)
    }

    pub fn replace_no_history_file<
        T: GetTmpPath + GetLiveVersionPath + Copy,
        U: FnMut(&mut fs::File) -> Result<(), DatabaseError>,
    >(
        &self,
        file: T,
        mut write_handle: U,
    ) -> Result<(), DatabaseError> {
        let tmp_file_path = self.git_repository_path.join(file.tmp_path().as_str());
        let mut tmp_file = fs::File::create(&tmp_file_path).map_err(DatabaseError::FileCreate)?;

        write_handle(&mut tmp_file)?;
        drop(tmp_file);

        let live_file_path = self.git_repository_path.join(file.live_path().as_str());
        fs::rename(&tmp_file_path, live_file_path).map_err(DatabaseError::FileRename)
    }

    pub fn read(&self) -> GitDatabaseReadCommands {
        GitDatabaseReadCommands::new(self.clone())
    }
}
