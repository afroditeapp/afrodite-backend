use async_trait::async_trait;
use error_stack::Result;
use tokio_stream::{Stream, StreamExt};
use tracing_subscriber::registry::Data;


use super::super::sqlite::{SqliteSelectJson, SqliteDatabaseError, SqliteReadHandle};
use crate::api::account::data::AccountSetup;
use crate::api::model::{Account, AccountId, Profile, AccountIdLight, ApiKey};
use crate::server::database::DatabaseError;
use crate::server::database::read::ReadCmd;
use crate::server::database::utils::GetReadWriteCmd;
use crate::utils::{IntoReportExt, ErrorConversion};

macro_rules! read_json {
    ($self:expr, $id:expr, $sql:literal, $str_field:ident) => {
        {
            let id = $id.as_uuid();
            sqlx::query!(
                $sql,
                id
            )
            .fetch_one($self.handle.pool())
            .await
            .into_error(SqliteDatabaseError::Execute)
            .and_then(|data|
                serde_json::from_str(&data.$str_field)
                    .into_error(SqliteDatabaseError::SerdeDeserialize)
                )
        }
    };
}

pub struct SqliteReadCommands<'a> {
    handle: &'a SqliteReadHandle,
}

impl<'a> SqliteReadCommands<'a> {
    pub fn new(handle: &'a SqliteReadHandle) -> Self {
        Self { handle }
    }

    pub async fn user_api_key(&self, id: &AccountId) -> Result<Option<ApiKey>, DatabaseError> {
        todo!()
    }

    pub async fn account_ids<T: FnMut(AccountIdLight)>(&self, mut handler: T) -> Result<(), DatabaseError> {
        let mut users = self.account_ids_stream();
        while let Some(user_id) = users.try_next().await.with_info(ReadCmd::Accounts)? {
            handler(user_id)
        }

        Ok(())
    }

    pub async fn read_json<
        T: SqliteSelectJson + GetReadWriteCmd
    >(&self, id: AccountIdLight) -> Result<T, DatabaseError> {
        T::select_json(id, self)
            .await
            .with_info_lazy(|| T::read_cmd(id))
    }

    fn account_ids_stream(&self) -> impl Stream<Item = Result<AccountIdLight, SqliteDatabaseError>> + '_ {
        sqlx::query!(
            r#"
            SELECT account_id
            FROM Account
            "#,
        )
        .fetch(self.handle.pool())
        .map(|result| {
            let result = result
                .into_error(SqliteDatabaseError::Fetch)?;
            AccountId::parse(result.account_id)
                .into_error(SqliteDatabaseError::Fetch)
                .map(|id| id.as_light())
        })
    }
}

#[async_trait]
impl SqliteSelectJson for Account {
    async fn select_json(
        id: AccountIdLight, read: &SqliteReadCommands,
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
impl SqliteSelectJson for AccountSetup {
    async fn select_json(
        id: AccountIdLight, read: &SqliteReadCommands,
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
impl SqliteSelectJson for Profile {
    async fn select_json(
        id: AccountIdLight, read: &SqliteReadCommands,
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
