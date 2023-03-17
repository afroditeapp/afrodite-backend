use async_trait::async_trait;
use error_stack::Result;
use tokio_stream::{Stream, StreamExt};

use super::super::sqlite::{SqliteDatabaseError, SqliteReadHandle};
use crate::api::account::data::AccountSetup;
use crate::api::model::{Account, AccountId, AccountIdLight, Profile};
use crate::server::database::sqlite::HistorySelectJson;
use crate::utils::IntoReportExt;

macro_rules! read_json {
    ($self:expr, $id:expr, $sql:literal, $str_field:ident) => {{
        let id = $id.as_uuid();
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

pub struct HistoryReadCommands<'a> {
    handle: &'a SqliteReadHandle,
}

impl<'a> HistoryReadCommands<'a> {
    pub fn new(handle: &'a SqliteReadHandle) -> Self {
        Self { handle }
    }

    pub fn account_ids(&self) -> impl Stream<Item = Result<AccountId, SqliteDatabaseError>> + '_ {
        sqlx::query!(
            r#"
            SELECT account_id
            FROM Account
            "#,
        )
        .fetch(self.handle.pool())
        .map(|result| {
            let result = result.into_error(SqliteDatabaseError::Fetch)?;
            AccountId::parse(result.account_id).into_error(SqliteDatabaseError::Fetch)
        })
    }
}

#[async_trait]
impl HistorySelectJson for Account {
    async fn history_select_json(
        id: AccountIdLight,
        read: &HistoryReadCommands,
    ) -> Result<Self, SqliteDatabaseError> {
        read_json!(
            read,
            id,
            r#"
            SELECT json_text
            FROM AccountState
            WHERE account_id = ?
            "#,
            json_text
        )
    }
}

#[async_trait]
impl HistorySelectJson for AccountSetup {
    async fn history_select_json(
        id: AccountIdLight,
        read: &HistoryReadCommands,
    ) -> Result<Self, SqliteDatabaseError> {
        read_json!(
            read,
            id,
            r#"
            SELECT json_text
            FROM AccountSetup
            WHERE account_id = ?
            "#,
            json_text
        )
    }
}

#[async_trait]
impl HistorySelectJson for Profile {
    async fn history_select_json(
        id: AccountIdLight,
        read: &HistoryReadCommands,
    ) -> Result<Self, SqliteDatabaseError> {
        read_json!(
            read,
            id,
            r#"
            SELECT json_text
            FROM Profile
            WHERE account_id = ?
            "#,
            json_text
        )
    }
}
