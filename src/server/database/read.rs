use error_stack::Result;
use tokio_stream::StreamExt;

use crate::{
    api::model::{
        Account,
        ApiKey, AccountId, AccountIdLight, Profile, AccountSetup,
    },
    utils::ErrorConversion,
};

use super::{
    git::{read::GitDatabaseReadCommands, utils::DatabasePath},
    sqlite::{read::SqliteReadCommands, SqliteReadHandle, utils::SqliteSelectJson},
    DatabaseError,
};



#[derive(Debug, Clone)]
pub enum ReadCmd {
    AccountApiKey(AccountId),
    AccountState(AccountId),
    AccountSetup(AccountId),
    Accounts,
    Profile(AccountId),
}

impl std::fmt::Display for ReadCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Read command: {:?}", self))
    }
}

pub struct ReadCommands<'a> {
    git_repositories: &'a DatabasePath,
    sqlite: SqliteReadCommands<'a>,
}

impl<'a> ReadCommands<'a> {
    pub fn new(git_repositories: &'a DatabasePath, sqlite: &'a SqliteReadHandle) -> Self {
        Self {
            git_repositories,
            sqlite: SqliteReadCommands::new(sqlite),
        }
    }

    pub async fn user_api_key(&self, user_id: &AccountId) -> Result<Option<ApiKey>, DatabaseError> {
        self.git(user_id)
            .api_key()
            .await
            .with_info_lazy(|| ReadCmd::AccountApiKey(user_id.clone()))
    }

    pub async fn account_ids<T: FnMut(AccountId)>(&self, mut handler: T) -> Result<(), DatabaseError> {
        let mut users = self.sqlite().account_ids();
        while let Some(user_id) = users.try_next().await.with_info(ReadCmd::Accounts)? {
            handler(user_id)
        }

        Ok(())
    }

    pub async fn read_json<
        T: SqliteSelectJson
    >(&self, id: &AccountId) -> Result<T, DatabaseError> {
        T::select_json(id, self.sqlite())
            .await
            .with_info_lazy(|| ReadCmd::AccountSetup(id.clone()))
    }

    pub(super) fn git(&self, user_id: &AccountId) -> GitDatabaseReadCommands {
        self.git_repositories.user_git_dir(user_id).read()
    }

    pub(super) fn sqlite(&self) -> &SqliteReadCommands {
        &self.sqlite
    }
}
