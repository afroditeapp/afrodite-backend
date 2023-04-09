use std::{
    fs,
    path::{Path, PathBuf},
};

use error_stack::Result;

use crate::api::model::{AccountId, AccountIdLight};

use super::{
    super::GitError,
    file::{GetStaticFileName},
    read::GitDatabaseReadCommands,
    GitDatabase,
};

use crate::utils::IntoReportExt;

/// Path to directory which contains all account data directories.
#[derive(Debug, Clone)]
pub struct FilesDir {
    database_dir: PathBuf,
}

impl FilesDir {
    pub fn new<T: AsRef<Path>>(database_dir: T) -> Self {
        Self {
            database_dir: database_dir.as_ref().to_path_buf(),
        }
    }

    pub fn user_git_dir(&self, id: &AccountIdLight) -> AccountFilesDir {
        AccountFilesDir {
            account_dir: self.database_dir.join(id.to_string()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.database_dir
    }
}

#[derive(Debug, Clone)]
pub struct AccountFilesDir {
    account_dir: PathBuf,
}

impl AccountFilesDir {
    /// Absolute path to profile directory
    pub fn path(&self) -> &PathBuf {
        &self.account_dir
    }

    pub fn exists(&self) -> bool {
        self.account_dir.exists()
    }


/*


    pub async fn read_to_string_optional<T: GetLiveVersionPath>(
        &self,
        file: T,
    ) -> Result<Option<String>, GitError> {
        let path = self.account_dir.join(file.live_path().as_str());
        if !path.is_file() {
            return Ok(None);
        }
        tokio::fs::read_to_string(path)
            .await
            .into_error_with_info(GitError::IoFileRead, file.live_path())
            .map(Some)
    }

    pub async fn read_to_string<T: GetLiveVersionPath>(&self, file: T) -> Result<String, GitError> {
        let path = self.account_dir.join(file.live_path().as_str());
        tokio::fs::read_to_string(path)
            .await
            .into_error_with_info(GitError::IoFileRead, file.live_path())
    }

    /// Open file for reading.
    pub fn open_file<T: GetLiveVersionPath>(&self, file: T) -> Result<fs::File, GitError> {
        let path = self.account_dir.join(file.live_path().as_str());
        fs::File::open(path).into_error_with_info(GitError::IoFileOpen, file.live_path())
    }

    /// Replace file using new file. Creates the file if it does not exists.
    pub fn replace_file<
        T: GetStaticFileName + GetLiveVersionPath + Copy,
        U: FnMut(&mut fs::File) -> Result<(), GitError>,
    >(
        &self,
        file: T,
        commit_msg: &str,
        mut write_handle: U,
    ) -> Result<(), GitError> {
        let git_file_path = self.account_dir.join(file.git_path().as_str());
        let mut git_file = fs::File::create(&git_file_path)
            .into_error_with_info_lazy(GitError::IoFileCreate, || {
                git_file_path.clone().to_string_lossy().to_string()
            })?;

        write_handle(&mut git_file)?;
        drop(git_file);

        let _git = GitDatabase::open(self)?;
        let _msg = match self.mode_msg.as_ref() {
            Some(mode_msg) => format!("{}\n\n{}", mode_msg, commit_msg),
            None => commit_msg.to_owned(),
        };

        let live_file_path = self.account_dir.join(file.live_path().as_str());
        fs::rename(&git_file_path, &live_file_path).into_error_with_info_lazy(
            GitError::IoFileRename,
            || {
                format!(
                    "from: {} to: {}",
                    git_file_path.to_string_lossy(),
                    live_file_path.to_string_lossy(),
                )
            },
        )
    }

    pub fn replace_no_history_file<
        T: GetTmpPath + GetLiveVersionPath + Copy,
        U: FnMut(&mut fs::File) -> Result<(), GitError>,
    >(
        &self,
        file: T,
        mut write_handle: U,
    ) -> Result<(), GitError> {
        let tmp_file_path = self.account_dir.join(file.tmp_path().as_str());
        let mut tmp_file = fs::File::create(&tmp_file_path)
            .into_error_with_info(GitError::IoFileCreate, file.tmp_path())?;

        write_handle(&mut tmp_file)?;
        drop(tmp_file);

        let live_file_path = self.account_dir.join(file.live_path().as_str());
        fs::rename(&tmp_file_path, &live_file_path).into_error_with_info_lazy(
            GitError::IoFileRename,
            || {
                format!(
                    "from: {} to: {}",
                    tmp_file_path.to_string_lossy(),
                    live_file_path.to_string_lossy(),
                )
            },
        )
    }

    pub fn read(&self) -> GitDatabaseReadCommands {
        GitDatabaseReadCommands::new(self.clone())
    }

 */
}
