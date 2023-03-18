use async_trait::async_trait;
use error_stack::Result;

use crate::{
    api::{
        account::data::AccountSetup,
        model::{Account, AccountIdInternal, Profile},
    },
    server::database::sqlite::HistoryUpdateJson,
};

use super::super::sqlite::{SqliteDatabaseError, SqliteWriteHandle};

use crate::utils::IntoReportExt;

macro_rules! insert_or_update_json {
    ($self:expr, $sql:literal, $data:expr, $id:expr) => {{
        let id = $id.as_uuid();
        let data = serde_json::to_string($data).into_error(SqliteDatabaseError::SerdeSerialize)?;
        sqlx::query!($sql, data, id,)
            .execute($self.handle.pool())
            .await
            .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }};
}

pub struct HistoryWriteCommands<'a> {
    handle: &'a SqliteWriteHandle,
}

impl<'a> HistoryWriteCommands<'a> {
    pub fn new(handle: &'a SqliteWriteHandle) -> Self {
        Self { handle }
    }

    pub async fn store_account_id(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), SqliteDatabaseError> {
        let id = id.as_uuid();
        sqlx::query!(
            r#"
            INSERT INTO AccountId (account_id)
            VALUES (?)
            "#,
            id
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }
}
