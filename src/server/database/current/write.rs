use async_trait::async_trait;
use error_stack::Result;

use crate::api::{
    account::data::AccountSetup,
    model::{Account, AccountIdInternal, Profile, AccountIdLight, ApiKey}, self,
};

use super::super::sqlite::{SqliteDatabaseError, SqliteUpdateJson, SqliteWriteHandle};

use crate::utils::IntoReportExt;

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

pub struct SqliteWriteCommands<'a> {
    handle: &'a SqliteWriteHandle,
}

impl<'a> SqliteWriteCommands<'a> {
    pub fn new(handle: &'a SqliteWriteHandle) -> Self {
        Self { handle }
    }

    pub async fn store_account_id(
        &self,
        id: AccountIdLight,
    ) -> Result<AccountIdInternal, SqliteDatabaseError> {
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
    ) -> Result<(), SqliteDatabaseError> {
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

    pub async fn store_profile(
        &self,
        id: AccountIdInternal,
        profile: &Profile,
    ) -> Result<(), SqliteDatabaseError> {
        insert_or_update_json!(
            self,
            r#"
            INSERT INTO Profile (json_text, account_row_id)
            VALUES (?, ?)
            "#,
            profile,
            id
        )
    }

    pub async fn store_account(
        &self,
        id: AccountIdInternal,
        account: &Account,
    ) -> Result<(), SqliteDatabaseError> {
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
    ) -> Result<(), SqliteDatabaseError> {
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
    ) -> Result<(), SqliteDatabaseError> {
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

#[async_trait]
impl SqliteUpdateJson for Account {
    async fn update_json(
        &self,
        id: AccountIdInternal,
        write: &SqliteWriteCommands,
    ) -> Result<(), SqliteDatabaseError> {
        insert_or_update_json!(
            write,
            r#"
            UPDATE Account
            SET json_text = ?
            WHERE account_row_id = ?
            "#,
            self,
            id
        )
    }
}

#[async_trait]
impl SqliteUpdateJson for AccountSetup {
    async fn update_json(
        &self,
        id: AccountIdInternal,
        write: &SqliteWriteCommands,
    ) -> Result<(), SqliteDatabaseError> {
        insert_or_update_json!(
            write,
            r#"
            UPDATE AccountSetup
            SET json_text = ?
            WHERE account_row_id = ?
            "#,
            self,
            id
        )
    }
}

#[async_trait]
impl SqliteUpdateJson for Profile {
    async fn update_json(
        &self,
        id: AccountIdInternal,
        write: &SqliteWriteCommands,
    ) -> Result<(), SqliteDatabaseError> {
        insert_or_update_json!(
            write,
            r#"
            UPDATE Profile
            SET json_text = ?
            WHERE account_row_id = ?
            "#,
            self,
            id
        )
    }
}
