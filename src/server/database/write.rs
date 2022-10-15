use std::{
    thread::sleep,
    time::Duration, io::Write,
};

use error_stack::{Result, ResultExt};

use tracing::error;

use crate::{
    server::database::{
        git::util::{DatabasePath, GitUserDirPath},
        DatabaseError, GitDatabaseOperationHandle, git::file::{CoreFile, CoreFileNoHistory}, sqlite::SqliteWriteHandle,
    }, api::core::{profile::Profile, user::{ApiKey, UserId}}, utils::{ErrorConversion},
};

use super::{git::write::GitDatabaseWriteCommands, sqlite::write::SqliteWriteCommands};

#[derive(Debug, Clone)]
pub enum WriteCmd {
    Register(UserId),
    UpdateProfile(UserId),
    UpdateApiKey(UserId),
}

impl std::fmt::Display for WriteCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Write command: {:?}", self))
    }
}


/// Write methods should be mutable to make sure that there is no concurrent
/// Git user directory writing.
pub struct WriteCommands {
    user_dir: GitUserDirPath,
    database_handle: GitDatabaseOperationHandle,
    sqlite_database_write: SqliteWriteHandle,
}

impl WriteCommands {
    pub fn new(
        user_dir: GitUserDirPath,
        database_handle: GitDatabaseOperationHandle,
        sqlite_database_write: SqliteWriteHandle,
    ) -> Self {
        Self {
            user_dir,
            database_handle,
            sqlite_database_write,
        }
    }

    pub async fn register(&mut self) -> Result<(), DatabaseError> {
        self.git().store_user_id().await.with_info_lazy(|| WriteCmd::Register(self.user_dir.id().clone()))?;
        self.sqlite().store_user_id(self.user_dir.id()).await.with_info_lazy(|| WriteCmd::Register(self.user_dir.id().clone()))

    }

    pub async fn update_user_profile(&mut self, profile_data: &Profile) -> Result<(), DatabaseError> {
        self.git().update_user_profile(profile_data).await
            .with_info_lazy(|| WriteCmd::UpdateProfile(self.user_dir.id().clone()))?;
        self.sqlite().update_user_profile(self.user_dir.id(), profile_data).await
            .with_info_lazy(|| WriteCmd::UpdateProfile(self.user_dir.id().clone()))
    }

    pub async fn update_current_api_key(&mut self, key: &ApiKey) -> Result<(), DatabaseError> {
        // Token is only stored as a file.
        self.git().update_token(key).await.with_info_lazy(|| WriteCmd::UpdateApiKey(self.user_dir.id().clone()))
    }

    fn git(&self) -> GitDatabaseWriteCommands {
        GitDatabaseWriteCommands::new(self.user_dir.clone(), self.database_handle.clone(), None)
    }

    /// Use constant title for commit messages when using write commands
    /// through returned object.
    pub(super) fn git_with_mode_message(&self, message: Option<&str>) -> GitDatabaseWriteCommands {
        GitDatabaseWriteCommands::new(self.user_dir.clone(), self.database_handle.clone(), message)
    }

    pub(super) fn sqlite(&self) -> SqliteWriteCommands {
        SqliteWriteCommands::new(&self.sqlite_database_write)
    }
}
