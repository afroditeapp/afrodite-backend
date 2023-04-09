
use std::fmt::Debug;

use tokio_stream::StreamExt;

use crate::{api::model::{AccountIdInternal, ApiKey, AccountIdLight}, utils::ErrorConversion};

use super::{current::read::SqliteReadCommands, sqlite::{SqliteReadHandle, SqliteSelectJson}, DatabaseError, utils::GetReadWriteCmd, cache::{ReadCacheJson, DatabaseCache}};

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
    cache: &'a DatabaseCache,
}

impl<'a> ReadCommands<'a> {
    pub fn new(sqlite: &'a SqliteReadHandle, cache: &'a DatabaseCache) -> Self {
        Self {
            sqlite: SqliteReadCommands::new(sqlite), cache
        }
    }

    pub async fn user_api_key(&self, id: AccountIdLight) -> Result<Option<ApiKey>, DatabaseError> {
        let id = self.cache.to_account_id_internal(id).await.change_context(DatabaseError::Cache)?;
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
        T: SqliteSelectJson + GetReadWriteCmd + ReadCacheJson
    >(&self, id: AccountIdInternal) -> Result<T, DatabaseError> {
        if T::CACHED_JSON {
            T::read_from_cache(id.as_light(), self.cache)
                .await
                .with_info_lazy(|| T::read_cmd(id))
        } else {
            T::select_json(id, &self.sqlite)
                .await
                .with_info_lazy(|| T::read_cmd(id))
        }
    }
}
