
use async_trait::async_trait;
use error_stack::Result;

use crate::{
    api::{
        account::data::AccountSetup,
        model::{Account, AccountIdInternal, Profile, AccountIdLight},
    },
    server::database::{sqlite::{HistoryUpdateJson, HistoryWriteHandle}, utils::current_unix_time},
};

use super::super::sqlite::{SqliteDatabaseError, SqliteWriteHandle};

use crate::utils::IntoReportExt;

macro_rules! insert_or_update_json {
    ($self:expr, $sql:literal, $data:expr, $unix_time:expr, $id:expr) => {{
        let id = $id.row_id();
        let unix_time = $unix_time;
        let data = serde_json::to_string($data).into_error(SqliteDatabaseError::SerdeSerialize)?;
        sqlx::query!($sql, data, unix_time, id,)
            .execute($self.handle.pool())
            .await
            .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }};
}

pub struct HistoryWriteCommands<'a> {
    handle: &'a HistoryWriteHandle,
}

impl<'a> HistoryWriteCommands<'a> {
    pub fn new(handle: &'a HistoryWriteHandle) -> Self {
        Self { handle }
    }

    pub async fn store_account_id(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), SqliteDatabaseError> {
        let row_id = id.row_id();
        let id = id.as_uuid();
        sqlx::query!(
            r#"
            INSERT INTO AccountId (account_row_id, account_id)
            VALUES (?, ?)
            "#,
            row_id,
            id
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
    ) -> Result<(), SqliteDatabaseError> {
        let unix_time = current_unix_time();
        insert_or_update_json!(
            self,
            r#"
            INSERT INTO HistoryAccount (json_text, unix_time, account_row_id)
            VALUES (?, ?, ?)
            "#,
            account,
            unix_time,
            id
        )
    }

    pub async fn store_account_setup(
        &self,
        id: AccountIdInternal,
        account: &AccountSetup,
    ) -> Result<(), SqliteDatabaseError> {
        let unix_time = current_unix_time();
        insert_or_update_json!(
            self,
            r#"
            INSERT INTO HistoryAccountSetup (json_text, unix_time, account_row_id)
            VALUES (?, ?, ?)
            "#,
            account,
            unix_time,
            id
        )
    }

    pub async fn store_profile(
        &self,
        id: AccountIdInternal,
        profile: &Profile,
    ) -> Result<(), SqliteDatabaseError> {
        let unix_time = current_unix_time();
        insert_or_update_json!(
            self,
            r#"
            INSERT INTO HistoryProfile (json_text, unix_time, account_row_id)
            VALUES (?, ?, ?)
            "#,
            profile,
            unix_time,
            id
        )
    }
}



#[async_trait]
impl HistoryUpdateJson for Account {
    async fn history_update_json(
        &self,
        id: AccountIdInternal,
        write: &HistoryWriteCommands,
    ) -> Result<(), SqliteDatabaseError> {
        write.store_account(id, self).await
    }
}


#[async_trait]
impl HistoryUpdateJson for AccountSetup {
    async fn history_update_json(
        &self,
        id: AccountIdInternal,
        write: &HistoryWriteCommands,
    ) -> Result<(), SqliteDatabaseError> {
        write.store_account_setup(id, self).await
    }
}


#[async_trait]
impl HistoryUpdateJson for Profile {
    async fn history_update_json(
        &self,
        id: AccountIdInternal,
        write: &HistoryWriteCommands,
    ) -> Result<(), SqliteDatabaseError> {
        write.store_profile(id, self).await
    }
}