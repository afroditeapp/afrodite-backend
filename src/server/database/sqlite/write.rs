use async_trait::async_trait;
use error_stack::Result;

use crate::api::{model::{Account, AccountId, AccountState, Profile}, account::data::AccountSetup};

use super::{SqliteDatabaseError, SqliteWriteHandle, utils::{SqliteUpdateJson,}};

use crate::utils::IntoReportExt;

macro_rules! insert_or_update_json {
    ($self:expr, $sql:literal, $data:expr, $id:expr) => {
        {
            let id = $id.as_str();
            let data =
                serde_json::to_string($data)
                    .into_error(SqliteDatabaseError::SerdeSerialize)?;
            sqlx::query!(
                $sql,
                data,
                id,
            )
            .execute($self.handle.pool())
            .await
            .into_error(SqliteDatabaseError::Execute)?;

            Ok(())
        }
    };
}

pub struct SqliteWriteCommands<'a> {
    handle: &'a SqliteWriteHandle,
}

impl<'a> SqliteWriteCommands<'a> {
    pub fn new(handle: &'a SqliteWriteHandle) -> Self {
        Self { handle }
    }

    pub async fn store_account_id(&mut self, id: &AccountId) -> Result<(), SqliteDatabaseError> {
        let id = id.as_str();
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

    pub async fn store_profile(&mut self, id: &AccountId, profile: &Profile) -> Result<(), SqliteDatabaseError> {
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

    pub async fn store_account(&mut self, id: &AccountId, account: &Account) -> Result<(), SqliteDatabaseError> {
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

    pub async fn store_account_setup(&mut self, id: &AccountId, account: &AccountSetup) -> Result<(), SqliteDatabaseError> {
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
impl SqliteUpdateJson for Account {
    async fn update_json(
        &self, id: &AccountId, write: &SqliteWriteCommands,
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
impl SqliteUpdateJson for AccountSetup {
    async fn update_json(
        &self, id: &AccountId, write: &SqliteWriteCommands,
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
impl SqliteUpdateJson for Profile {
    async fn update_json(
        &self, id: &AccountId, write: &SqliteWriteCommands,
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
