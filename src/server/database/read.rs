
use tokio_stream::StreamExt;

use crate::{api::model::{AccountIdInternal, ApiKey}, utils::ErrorConversion};

use super::{current::read::SqliteReadCommands, sqlite::{SqliteReadHandle, SqliteSelectJson}, DatabaseError, utils::GetReadWriteCmd};

use error_stack::{Result, ResultExt};

#[derive(Debug, Clone)]
pub enum ReadCmd {
    AccountApiKey(AccountIdInternal),
    AccountState(AccountIdInternal),
    AccountSetup(AccountIdInternal),
    Accounts,
    Profile(AccountIdInternal),
}

impl std::fmt::Display for ReadCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Read command: {:?}", self))
    }
}

pub struct ReadCommands<'a> {
    sqlite: SqliteReadCommands<'a>,
}

impl<'a> ReadCommands<'a> {
    pub fn new(sqlite: &'a SqliteReadHandle) -> Self {
        Self {
            sqlite: SqliteReadCommands::new(sqlite),
        }
    }

    pub async fn user_api_key(&self, id: AccountIdInternal) -> Result<Option<ApiKey>, DatabaseError> {
        self.sqlite.api_key(id).await
            .change_context(DatabaseError::Sqlite)
    }

    pub async fn account_ids<T: FnMut(AccountIdInternal)>(&self, mut handler: T) -> Result<(), DatabaseError> {
        let mut users = self.sqlite.account_ids_stream();
        while let Some(user_id) = users.try_next().await.with_info(ReadCmd::Accounts)? {
            handler(user_id)
        }

        Ok(())
    }

    pub async fn read_json<
        T: SqliteSelectJson + GetReadWriteCmd
    >(&self, id: AccountIdInternal) -> Result<T, DatabaseError> {
        T::select_json(id, &self.sqlite)
            .await
            .with_info_lazy(|| T::read_cmd(id))
    }

}
