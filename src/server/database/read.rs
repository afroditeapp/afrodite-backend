use std::{collections::HashMap, marker};

use crate::{
    api::core::{user::{UserId, ApiKey}, profile::Profile},
    server::database::{DatabaseError}
};

use super::{git::{util::{GitUserDirPath, DatabasePath}, GitDatabaseOperationHandle, read::GitDatabaseReadCommands, GitDatabase, write::GitDatabaseWriteCommands, file::CoreFileNoHistory}, sqlite::{SqliteReadHandle, SqliteWriteHandle, read::SqliteReadCommands}};

pub struct ReadCommands<'a> {
    git_repositories: &'a DatabasePath,
    sqlite: &'a SqliteReadHandle,
}

impl <'a> ReadCommands<'a> {
    pub fn new(
        git_repositories: &'a DatabasePath,
        sqlite: &'a SqliteReadHandle,
    ) -> Self {
        Self {
            git_repositories,
            sqlite,
        }
    }

    pub async fn user_api_key(&self, user_id: &UserId) -> Result<Option<ApiKey>, DatabaseError> {
        self.git(user_id).api_key().await
    }

    pub async fn users<T: FnMut(UserId)>(&self, handler: T) -> Result<(), DatabaseError> {
        self.sqlite().users(handler).await
    }

    pub async fn user_profile(&self, user_id: &UserId) -> Result<Profile, DatabaseError> {
        self.sqlite().user_profile(user_id).await
    }

    fn git(&self, user_id: &UserId) -> GitDatabaseReadCommands {
        self.git_repositories.user_git_dir(user_id).read()
    }

    fn sqlite(&self) -> SqliteReadCommands {
        SqliteReadCommands::new(&self.sqlite)
    }
}
