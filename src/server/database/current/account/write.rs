use async_trait::async_trait;
use error_stack::Result;

use crate::api::account::data::AccountSetup;
use crate::server::database::current::CurrentDataWriteCommands;
use crate::server::database::sqlite::{SqliteDatabaseError, SqliteUpdateJson};

use crate::api::model::*;

use crate::utils::IntoReportExt;

use crate::insert_or_update_json;



use crate::{
    api::{
        model::{AccountIdInternal},
    },
    server::database::{sqlite::CurrentDataWriteHandle, write::WriteResult},
};

pub struct CurrentWriteAccountCommands<'a> {
    handle: &'a CurrentDataWriteHandle,
}

impl<'a> CurrentWriteAccountCommands<'a> {
    pub fn new(handle: &'a CurrentDataWriteHandle) -> Self {
        Self { handle }
    }

    pub async fn store_account_id(
        &self,
        id: AccountIdLight,
    ) -> WriteResult<AccountIdInternal, SqliteDatabaseError, AccountIdLight> {
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
    ) -> WriteResult<(), SqliteDatabaseError, ApiKey> {
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

    pub async fn store_refresh_token(
        &self,
        id: AccountIdInternal,
        refresh_token: Option<RefreshToken>,
    ) -> WriteResult<(), SqliteDatabaseError, ApiKey> {
        let refresh_token = if let Some(t) = refresh_token {
            Some(
                t.bytes()
                    .into_error(SqliteDatabaseError::DataFormatConversion)?,
            )
        } else {
            None
        };
        let id = id.row_id();
        sqlx::query!(
            r#"
            INSERT INTO RefreshToken (refresh_token, account_row_id)
            VALUES (?, ?)
            "#,
            refresh_token,
            id,
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
    ) -> WriteResult<(), SqliteDatabaseError, Account> {
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
    ) -> WriteResult<(), SqliteDatabaseError, AccountSetup> {
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

    pub async fn store_sign_in_with_info(
        &self,
        id: AccountIdInternal,
        sign_in_with_info: &SignInWithInfo,
    ) -> WriteResult<(), SqliteDatabaseError, SignInWithInfo> {
        let id = id.row_id();
        sqlx::query!(
            r#"
            INSERT INTO SignInWithInfo (google_account_id, account_row_id)
            VALUES (?, ?)
            "#,
            sign_in_with_info.google_account_id,
            id,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn update_api_key(
        &self,
        id: AccountIdInternal,
        api_key: Option<&ApiKey>,
    ) -> WriteResult<(), SqliteDatabaseError, ApiKey> {
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

    pub async fn update_refresh_token(
        &self,
        id: AccountIdInternal,
        refresh_token: Option<&RefreshToken>,
    ) -> WriteResult<(), SqliteDatabaseError, ApiKey> {
        let refresh_token = if let Some(t) = refresh_token {
            Some(
                t.bytes()
                    .into_error(SqliteDatabaseError::DataFormatConversion)?,
            )
        } else {
            None
        };
        let id = id.row_id();
        sqlx::query!(
            r#"
            UPDATE RefreshToken
            SET refresh_token = ?
            WHERE account_row_id = ?
            "#,
            refresh_token,
            id,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn update_sign_in_with_info(
        &self,
        id: AccountIdInternal,
        sign_in_with: &SignInWithInfo,
    ) -> WriteResult<(), SqliteDatabaseError, ApiKey> {
        let id = id.row_id();
        sqlx::query!(
            r#"
            UPDATE SignInWithInfo
            SET google_account_id = ?
            WHERE account_row_id = ?
            "#,
            sign_in_with.google_account_id,
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
        write: &CurrentDataWriteCommands,
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
        write: &CurrentDataWriteCommands,
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
