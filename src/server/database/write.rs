use std::{
    thread::sleep,
    time::Duration, io::Write,
};

use tracing::error;

use crate::{
    server::database::{
        git::util::{DatabasePath, GitUserDirPath},
        DatabaseError, GitDatabaseOperationHandle, git::file::{CoreFile, CoreFileNoHistory}, sqlite::SqliteWriteHandle,
    }, api::core::{profile::Profile, user::ApiKey},
};

use super::{git::write::GitDatabaseWriteCommands, sqlite::write::SqliteWriteCommands};


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
        self.git().store_user_id().await?;
        self.sqlite().store_user_id(self.user_dir.id()).await

    }

    pub async fn update_user_profile(&mut self, profile_data: &Profile) -> Result<(), DatabaseError> {
        self.git().update_user_profile(profile_data).await?;
        self.sqlite().update_user_profile(self.user_dir.id(), profile_data).await
    }

    pub async fn update_current_api_key(&mut self, key: &ApiKey) -> Result<(), DatabaseError> {
        // Token is only stored as a file.
        self.git().update_token(key).await
    }

    fn git(&self) -> GitDatabaseWriteCommands {
        GitDatabaseWriteCommands::new(self.user_dir.clone(), self.database_handle.clone())
    }

    fn sqlite(&self) -> SqliteWriteCommands {
        SqliteWriteCommands::new(&self.sqlite_database_write)
    }
}
