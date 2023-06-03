pub mod account;
pub mod media;
pub mod profile;

use error_stack::IntoReport;
use tokio_stream::{Stream, StreamExt};

use self::media::admin_write::CurrentWriteMediaAdminCommands;
use self::media::read::CurrentReadMediaCommands;
use self::media::write::CurrentWriteMediaCommands;
use self::profile::read::CurrentReadProfileCommands;
use self::profile::write::CurrentWriteProfileCommands;

use super::read::ReadResult;
use super::sqlite::CurrentDataWriteHandle;
use super::write::{NoId, WriteResult};
use crate::api::account::data::AccountSetup;
use crate::server::database::sqlite::{SqliteDatabaseError, SqliteReadHandle};

use crate::api::model::{Account, AccountIdInternal, AccountIdLight, ApiKey, RefreshToken, SignInWithInfo, GoogleAccountId};

use crate::utils::IntoReportExt;

#[macro_export]
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

#[macro_export]
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

pub struct SqliteReadCommands<'a> {
    handle: &'a SqliteReadHandle,
}

impl<'a> SqliteReadCommands<'a> {
    pub fn new(handle: &'a SqliteReadHandle) -> Self {
        Self { handle }
    }

    pub fn media(&self) -> CurrentReadMediaCommands<'_> {
        CurrentReadMediaCommands::new(self.handle)
    }

    pub fn profile(&self) -> CurrentReadProfileCommands<'_> {
        CurrentReadProfileCommands::new(self.handle)
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

pub struct CurrentDataWriteCommands<'a> {
    handle: &'a CurrentDataWriteHandle,
}

impl<'a> CurrentDataWriteCommands<'a> {
    pub fn new(handle: &'a CurrentDataWriteHandle) -> Self {
        Self { handle }
    }

    pub fn media(self) -> CurrentWriteMediaCommands<'a> {
        CurrentWriteMediaCommands::new(self.handle)
    }

    pub fn media_admin(self) -> CurrentWriteMediaAdminCommands<'a> {
        CurrentWriteMediaAdminCommands::new(self.handle)
    }

    pub fn profile(self) -> CurrentWriteProfileCommands<'a> {
        CurrentWriteProfileCommands::new(self.handle)
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
            Some(t.bytes().into_error(SqliteDatabaseError::DataFormatConversion)?)
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
            Some(t.bytes().into_error(SqliteDatabaseError::DataFormatConversion)?)
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
