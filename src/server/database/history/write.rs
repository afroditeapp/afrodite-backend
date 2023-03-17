use async_trait::async_trait;
use error_stack::Result;

use crate::{
    api::{
        account::data::AccountSetup,
        model::{Account, AccountIdLight, Profile},
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
        &mut self,
        id: AccountIdLight,
    ) -> Result<(), SqliteDatabaseError> {
        let id = id.as_uuid();
        sqlx::query!(
            r#"
            INSERT INTO Account (account_id)
            VALUES (?)
            "#,
            id
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn store_profile(
        &mut self,
        id: AccountIdLight,
        profile: &Profile,
    ) -> Result<(), SqliteDatabaseError> {
        insert_or_update_json!(
            self,
            r#"
            INSERT INTO Profile (json_text, account_id)
            VALUES (?, ?)
            "#,
            profile,
            id
        )
    }

    pub async fn store_account(
        &mut self,
        id: AccountIdLight,
        account: &Account,
    ) -> Result<(), SqliteDatabaseError> {
        insert_or_update_json!(
            self,
            r#"
            INSERT INTO AccountState (json_text, account_id)
            VALUES (?, ?)
            "#,
            account,
            id
        )
    }

    pub async fn store_account_setup(
        &mut self,
        id: AccountIdLight,
        account: &AccountSetup,
    ) -> Result<(), SqliteDatabaseError> {
        insert_or_update_json!(
            self,
            r#"
            INSERT INTO AccountSetup (json_text, account_id)
            VALUES (?, ?)
            "#,
            account,
            id
        )
    }
}

#[async_trait]
impl HistoryUpdateJson for Account {
    async fn history_update_json(
        &self,
        id: AccountIdLight,
        write: &HistoryWriteCommands,
    ) -> Result<(), SqliteDatabaseError> {
        insert_or_update_json!(
            write,
            r#"
            UPDATE AccountState
            SET json_text = ?
            WHERE account_id = ?
            "#,
            self,
            id
        )
    }
}

#[async_trait]
impl HistoryUpdateJson for AccountSetup {
    async fn history_update_json(
        &self,
        id: AccountIdLight,
        write: &HistoryWriteCommands,
    ) -> Result<(), SqliteDatabaseError> {
        insert_or_update_json!(
            write,
            r#"
            UPDATE AccountSetup
            SET json_text = ?
            WHERE account_id = ?
            "#,
            self,
            id
        )
    }
}

#[async_trait]
impl HistoryUpdateJson for Profile {
    async fn history_update_json(
        &self,
        id: AccountIdLight,
        write: &HistoryWriteCommands,
    ) -> Result<(), SqliteDatabaseError> {
        insert_or_update_json!(
            write,
            r#"
            UPDATE Profile
            SET json_text = ?
            WHERE account_id = ?
            "#,
            self,
            id
        )
    }
}
