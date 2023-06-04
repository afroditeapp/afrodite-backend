use async_trait::async_trait;
use error_stack::Result;
use futures::Stream;

use crate::api::account::data::AccountSetup;
use crate::server::database::current::SqliteReadCommands;
use crate::server::database::sqlite::{SqliteDatabaseError, SqliteSelectJson};

use crate::api::model::*;

use crate::server::database::write::NoId;
use crate::utils::IntoReportExt;

use std::collections::HashSet;

use tokio_stream::StreamExt;

use super::super::super::sqlite::{SqliteReadHandle};

use crate::api::media::data::{
    ContentIdInternal, ContentState, Moderation, ModerationId, ModerationRequestId,
    ModerationRequestQueueNumber, ModerationRequestState,
};
use crate::api::model::{
    AccountIdInternal, AccountIdLight, ContentId, ModerationRequestContent,
    ModerationRequestInternal,
};
use crate::server::database::file::file::ImageSlot;

use crate::server::database::read::ReadResult;


use crate::read_json;


pub struct CurrentReadAccountCommands<'a> {
    handle: &'a SqliteReadHandle,
}

impl<'a> CurrentReadAccountCommands<'a> {
    pub fn new(handle: &'a SqliteReadHandle) -> Self {
        Self { handle }
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
        .map(|result| {
            result
                .into_error(SqliteDatabaseError::Fetch)
                .map_err(|e| e.into())
        })
    }

    pub async fn access_token(
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

    pub async fn refresh_token(
        &self,
        id: AccountIdInternal,
    ) -> ReadResult<Option<RefreshToken>, SqliteDatabaseError, RefreshToken> {
        let id = id.row_id();
        sqlx::query!(
            r#"
            SELECT refresh_token
            FROM RefreshToken
            WHERE account_row_id = ?
            "#,
            id
        )
        .fetch_one(self.handle.pool())
        .await
        .map(|result| result.refresh_token.as_deref().map(RefreshToken::from_bytes))
        .into_error(SqliteDatabaseError::Fetch)
        .map_err(|e| e.into())
    }

    pub async fn sign_in_with_info(
        &self,
        id: AccountIdInternal,
    ) -> ReadResult<SignInWithInfo, SqliteDatabaseError> {
        let id = id.row_id();
        sqlx::query_as!(
            SignInWithInfo,
            r#"
            SELECT google_account_id as "google_account_id: _"
            FROM SignInWithInfo
            WHERE account_row_id = ?
            "#,
            id
        )
        .fetch_one(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)
        .map_err(|e| e.into())
    }

    pub async fn get_account_with_google_account_id(
        &self,
        google_account_id: GoogleAccountId,
    ) -> ReadResult<Option<AccountIdInternal>, SqliteDatabaseError> {
        sqlx::query!(
            r#"
            SELECT AccountId.account_row_id, AccountId.account_id as "account_id: uuid::Uuid"
            FROM SignInWithInfo
            INNER JOIN AccountId on AccountId.account_row_id = SignInWithInfo.account_row_id
            WHERE google_account_id = ?
            "#,
            google_account_id
        )
        .fetch_optional(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)
        .map_err(|e| e.into())
        .map(|r| r.map(|r| AccountIdInternal { account_id: r.account_id, account_row_id: r.account_row_id }))
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
