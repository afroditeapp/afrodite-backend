use error_stack::Result;
use serde::Serialize;
use sqlx::Sqlite;

use crate::{
    api::model::{
        Account,
        ApiKey, AccountId, AccountState, Profile, AccountSetup,
    },
    server::database::{
        git::utils::GitUserDirPath, sqlite::SqliteWriteHandle, DatabaseError,
        GitDatabaseOperationHandle,
    },
    utils::ErrorConversion, config::Config,
};

use super::{git::{write::GitDatabaseWriteCommands, file::GitJsonFile}, sqlite::{write::SqliteWriteCommands, SqliteDatabaseError, utils::SqliteUpdateJson}, utils::GetReadWriteCmd};

#[derive(Debug, Clone)]
pub enum WriteCmd {
    AccountId(AccountId),
    Profile(AccountId),
    ApiKey(AccountId),
    AccountState(AccountId),
    AccountSetup(AccountId),
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

    pub async fn register(&mut self, config: &Config) -> Result<(), DatabaseError> {
        let account_state = Account::default();
        let account_setup = AccountSetup::default();
        let profile = Profile::default();

        self.git()
            .store_account_id()
            .await
            .with_info_lazy(|| WriteCmd::AccountId(self.user_dir.id().clone()))?;

        if config.components().account {
            self.git()
                .update_json(&account_state)
                .await
                .with_write_cmd_info::<Account>(self.user_dir.id())?;

            self.git()
                .update_json(&account_setup)
                .await
                .with_write_cmd_info::<AccountSetup>(self.user_dir.id())?;
        }

        if config.components().profile {
            self.git()
                .update_json(&profile)
                .await
                .with_write_cmd_info::<Profile>(self.user_dir.id())?;
        }

        self.sqlite()
            .store_account_id(self.user_dir.id())
            .await
            .with_info_lazy(|| WriteCmd::AccountId(self.user_dir.id().clone()))?;

        if config.components().account {
            self.sqlite()
                .store_account(self.user_dir.id(), &account_state)
                .await
                .with_write_cmd_info::<Account>(self.user_dir.id())?;

            self.sqlite()
                .store_account_setup(self.user_dir.id(), &account_setup)
                .await
                .with_write_cmd_info::<AccountSetup>(self.user_dir.id())?;
        }

        if config.components().profile {
            self.sqlite()
                .store_profile(self.user_dir.id(), &profile)
                .await
                .with_write_cmd_info::<Profile>(self.user_dir.id())?;
        }

        Ok(())
    }

    pub async fn update_current_api_key(&mut self, key: &ApiKey) -> Result<(), DatabaseError> {
        // Token is only stored as a file.
        self.git()
            .update_token(key)
            .await
            .with_info_lazy(|| WriteCmd::ApiKey(self.user_dir.id().clone()))
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

    pub async fn update_json<
        T: GetReadWriteCmd + Serialize + Clone + Send + GitJsonFile + SqliteUpdateJson + 'static
    >(
        &mut self,
        data: &T,
    ) -> Result<(), DatabaseError> {
        self.git()
            .update_json(data)
            .await
            .with_write_cmd_info::<T>(self.user_dir.id())?;
        data.update_json(self.user_dir.id(), &self.sqlite())
            .await
            .with_write_cmd_info::<T>(self.user_dir.id())
    }
}
