pub mod media;

use api_client::models::new_moderation_request;
use async_trait::async_trait;
use error_stack::Result;
use tokio_stream::{Stream, StreamExt};

use self::media::CurrentReadMediaCommands;

use super::super::sqlite::{SqliteDatabaseError, SqliteReadHandle, SqliteSelectJson};
use crate::api::account::data::AccountSetup;
use crate::api::media::data::ModerationRequestState;
use crate::api::model::{Account, AccountId, AccountIdInternal, ApiKey, Profile, ModerationRequest};
use crate::server::database::read::ReadCmd;
use crate::server::database::utils::GetReadWriteCmd;
use crate::server::database::DatabaseError;
use crate::utils::{ErrorConversion, IntoReportExt};

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

    pub fn account_ids_stream(
        &self,
    ) -> impl Stream<Item = Result<AccountIdInternal, SqliteDatabaseError>> + '_ {
        sqlx::query_as!(
            AccountIdInternal,
            r#"
            SELECT account_row_id, account_id as "account_id: _"
            FROM AccountId
            "#,
        )
        .fetch(self.handle.pool())
        .map(|result| {
            result.into_error(SqliteDatabaseError::Fetch)
        })
    }

    pub async fn api_key(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<ApiKey>, SqliteDatabaseError> {
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
        .map(|result| {
            result
                .api_key
                .map(ApiKey::new)
        })
        .into_error(SqliteDatabaseError::Fetch)
    }
}

#[async_trait]
impl SqliteSelectJson for Account {
    async fn select_json(
        id: AccountIdInternal,
        read: &SqliteReadCommands,
    ) -> Result<Self, SqliteDatabaseError> {
        read_json!(
            read,
            id,
            r#"
            SELECT json_text
            FROM Account
            WHERE account_row_id = ?
            "#,
            json_text
        )
    }
}

#[async_trait]
impl SqliteSelectJson for AccountSetup {
    async fn select_json(
        id: AccountIdInternal,
        read: &SqliteReadCommands,
    ) -> Result<Self, SqliteDatabaseError> {
        read_json!(
            read,
            id,
            r#"
            SELECT json_text
            FROM AccountSetup
            WHERE account_row_id = ?
            "#,
            json_text
        )
    }
}

#[async_trait]
impl SqliteSelectJson for Profile {
    async fn select_json(
        id: AccountIdInternal,
        read: &SqliteReadCommands,
    ) -> Result<Self, SqliteDatabaseError> {
        read_json!(
            read,
            id,
            r#"
            SELECT json_text
            FROM Profile
            WHERE account_row_id = ?
            "#,
            json_text
        )
    }
}
