use api_client::models::new_moderation_request;
use async_trait::async_trait;
use error_stack::Result;
use tokio_stream::{Stream, StreamExt};

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

    pub async fn current_media_moderation_request(
        &self,
        id_internal: AccountIdInternal,
    ) -> Result<Option<ModerationRequest>, SqliteDatabaseError> {
        let id = id_internal.row_id();
        sqlx::query!(
            r#"
            SELECT row_id, state_number, json_text
            FROM MediaModerationRequest
            WHERE account_row_id = ?
            "#,
            id
        )
        .fetch_optional(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)
        .and_then(|result| {
            if let Some(r) = result{
                serde_json::from_str(&r.json_text)
                    .into_error(SqliteDatabaseError::SerdeDeserialize)
                    .and_then(
                        |new_moderation_request| {
                            match r.state_number.try_into() {
                                Ok(state) => Ok((new_moderation_request, state)),
                                Err(e) => Err(e).into_error(SqliteDatabaseError::TryFromError),
                            }
                        }
                    )
                    .map(
                        |(new_moderation_request, state): (_, ModerationRequestState)| {
                            ModerationRequest::new(
                                r.row_id,
                                 id_internal.as_light(),
                                  state,
                                   new_moderation_request,
                                ).into()
                        }
                    )
            } else {
                Ok(None)
            }
        })
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
