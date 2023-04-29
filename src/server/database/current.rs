pub mod account;
pub mod media;
pub mod profile;


use async_trait::async_trait;
use error_stack::Result;
use tokio_stream::{Stream, StreamExt};

use self::media::admin_write::CurrentWriteMediaAdminCommands;
use self::media::read::CurrentReadMediaCommands;
use self::media::write::CurrentWriteMediaCommands;
use self::profile::read::CurrentReadProfileCommands;
use self::profile::write::CurrentWriteProfileCommands;

use crate::server::database::sqlite::{SqliteDatabaseError, SqliteReadHandle, SqliteSelectJson};
use super::read::ReadResult;
use super::sqlite::CurrentDataWriteHandle;
use super::write::{WriteError, WriteResult, NoId};
use crate::api::account::data::AccountSetup;

use crate::api::model::{
    Account, AccountIdInternal, ApiKey, Profile, AccountIdLight,
};



use crate::utils::{IntoReportExt};

#[macro_export]
macro_rules! read_json {
    ($self:expr, $id:expr, $sql:literal, $str_field:ident) => {{
        let id = $id.row_id();
        sqlx::query!($sql, id)
            .fetch_one($self.handle.pool())
            .await
            .into_error(SqliteDatabaseError::Execute)
            .and_then(|data| {
                serde_json::from_str(&data.$str_field)
                    .into_error(SqliteDatabaseError::SerdeDeserialize)
            })
    }};
}

#[macro_export]
macro_rules! insert_or_update_json {
    ($self:expr, $sql:literal, $data:expr, $id:expr) => {{
        let id = $id.row_id();
        let data = serde_json::to_string($data).into_error(SqliteDatabaseError::SerdeSerialize)?;
        sqlx::query!($sql, data, id)
            .execute($self.handle.pool())
            .await
            .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }};
}

pub struct SqliteReadCommands<'a> {
    handle: &'a SqliteReadHandle,
}

impl<'a> SqliteReadCommands<'a> {
    pub fn new(handle: &'a SqliteReadHandle) -> Self {
        Self { handle }
    }

    pub fn media(&self) -> CurrentReadMediaCommands<'_> {
        CurrentReadMediaCommands::new(self.handle)
    }

    pub fn profile(&self) -> CurrentReadProfileCommands<'_> {
        CurrentReadProfileCommands::new(self.handle)
    }

    pub fn account_ids_stream(
        &self,
    ) -> impl Stream<Item = ReadResult<AccountIdInternal, SqliteDatabaseError, NoId>> + '_ {
        sqlx::query_as!(
            AccountIdInternal,
            r#"
            SELECT account_row_id, account_id as "account_id: _"
            FROM AccountId
            "#,
        )
        .fetch(self.handle.pool())
        .map(|result| result.into_error(SqliteDatabaseError::Fetch).map_err(|e| e.into()))
    }

    pub async fn api_key(
        &self,
        id: AccountIdInternal,
    ) -> ReadResult<Option<ApiKey>, SqliteDatabaseError, ApiKey> {
        let id = id.row_id();
        sqlx::query!(
            r#"
            SELECT api_key
            FROM ApiKey
            WHERE account_row_id = ?
            "#,
            id
        )
        .fetch_one(self.handle.pool())
        .await
        .map(|result| result.api_key.map(ApiKey::new))
        .into_error(SqliteDatabaseError::Fetch)
        .map_err(|e| e.into())
    }
}

pub struct CurrentDataWriteCommands<'a> {
    handle: &'a CurrentDataWriteHandle,
}

impl<'a> CurrentDataWriteCommands<'a> {
    pub fn new(handle: &'a CurrentDataWriteHandle) -> Self {
        Self { handle }
    }

    pub fn media(self) -> CurrentWriteMediaCommands<'a> {
        CurrentWriteMediaCommands::new(self.handle)
    }

    pub fn media_admin(self) -> CurrentWriteMediaAdminCommands<'a> {
        CurrentWriteMediaAdminCommands::new(self.handle)
    }

    pub fn profile(self) -> CurrentWriteProfileCommands<'a> {
        CurrentWriteProfileCommands::new(self.handle)
    }

    pub async fn store_account_id(
        &self,
        id: AccountIdLight,
    ) -> WriteResult<AccountIdInternal, SqliteDatabaseError, AccountIdLight> {
        let id = id.as_uuid();
        let insert_result = sqlx::query!(
            r#"
            INSERT INTO AccountId (account_id)
            VALUES (?)
            "#,
            id
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(AccountIdInternal {
            account_id: id,
            account_row_id: insert_result.last_insert_rowid(),
        })
    }

    pub async fn store_api_key(
        &self,
        id: AccountIdInternal,
        api_key: Option<ApiKey>,
    ) -> WriteResult<(), SqliteDatabaseError, ApiKey> {
        let api_key = api_key.as_ref().map(|k| k.as_str());
        let id = id.row_id();
        sqlx::query!(
            r#"
            INSERT INTO ApiKey (api_key, account_row_id)
            VALUES (?, ?)
            "#,
            api_key,
            id,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }


    pub async fn store_account(
        &self,
        id: AccountIdInternal,
        account: &Account,
    ) -> WriteResult<(), SqliteDatabaseError, Account> {
        insert_or_update_json!(
            self,
            r#"
            INSERT INTO Account (json_text, account_row_id)
            VALUES (?, ?)
            "#,
            account,
            id
        )
    }

    pub async fn store_account_setup(
        &self,
        id: AccountIdInternal,
        account: &AccountSetup,
    ) -> WriteResult<(), SqliteDatabaseError, AccountSetup> {
        insert_or_update_json!(
            self,
            r#"
            INSERT INTO AccountSetup (json_text, account_row_id)
            VALUES (?, ?)
            "#,
            account,
            id
        )
    }

    pub async fn update_api_key(
        &self,
        id: AccountIdInternal,
        api_key: Option<&ApiKey>,
    ) -> WriteResult<(), SqliteDatabaseError, ApiKey> {
        let api_key = api_key.as_ref().map(|k| k.as_str());
        let id = id.row_id();
        sqlx::query!(
            r#"
            UPDATE ApiKey
            SET api_key = ?
            WHERE account_row_id = ?
            "#,
            api_key,
            id,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }
}
