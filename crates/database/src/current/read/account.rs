use diesel::prelude::*;
use async_trait::async_trait;
use futures::Stream;
use model::{
    Account, AccountIdInternal, AccountSetup, ApiKey, GoogleAccountId, RefreshToken, SignInWithInfo, RefreshTokenRaw, AccessTokenRaw, AccountIdDb, AccountIdLight, AccountRaw, SignInWithInfoRaw,
};

use tokio_stream::StreamExt;
use utils::IntoReportExt;
use error_stack::Result;

use crate::{
    IntoDatabaseError,
    current::read::SqliteReadCommands,
    read_json,
    sqlite::{SqliteDatabaseError, SqliteSelectJson},
    NoId, ReadResult, diesel::DieselDatabaseError, ReadError,
};

define_read_commands!(CurrentReadAccount, CurrentSyncReadAccount);

impl CurrentReadAccount<'_> {
    pub fn account_ids_stream(
        &self,
    ) -> impl Stream<Item = Result<AccountIdInternal, SqliteDatabaseError>> + '_ {
        sqlx::query!(
            r#"
            SELECT id, uuid as "account_id: uuid::Uuid"
            FROM account_id
            "#,
        )
        .fetch(self.pool())
        .map(|result| {
            result
                .map(|data| {
                    let id = AccountIdDb::new(data.id);
                    let account_id = AccountIdLight::new(data.account_id);
                    AccountIdInternal::new(id, account_id)
                })
                .into_db_error(SqliteDatabaseError::Fetch, ())
        })
    }
}

impl <'a> CurrentSyncReadAccount<'a> {
    pub fn google_account_id_to_account_id(
        &'a mut self,
        google_id: GoogleAccountId,
    ) -> Result<AccountIdInternal, DieselDatabaseError> {
        use crate::schema::account_id;
        use crate::schema::sign_in_with_info;

        sign_in_with_info::table
            .inner_join(account_id::table)
            .filter(sign_in_with_info::google_account_id.eq(google_id.as_str()))
            .select(AccountIdInternal::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, google_id)
    }

    pub fn sign_in_with_info(
        &'a mut self,
        id: AccountIdInternal,
    ) -> Result<SignInWithInfo, DieselDatabaseError> {
        use crate::schema::sign_in_with_info::dsl::*;

        sign_in_with_info
            .filter(account_id.eq(id.as_db_id()))
            .select(SignInWithInfoRaw::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)
            .map(Into::into)
    }

    pub fn refresh_token(
        &'a mut self,
        id: AccountIdInternal,
    ) -> Result<Option<RefreshToken>, DieselDatabaseError> {
        use crate::schema::refresh_token::dsl::*;

        let raw = refresh_token
            .filter(account_id.eq(id.as_db_id()))
            .select(RefreshTokenRaw::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        if let Some(data) = raw.token {
            Ok(Some(RefreshToken::from_bytes(&data)))
        } else {
            Ok(None)
        }
    }

    pub fn access_token(
        &'a mut self,
        id: AccountIdInternal,
    ) -> Result<Option<ApiKey>, DieselDatabaseError> {
        use crate::schema::access_token::dsl::*;

        let raw = access_token
            .filter(account_id.eq(id.as_db_id()))
            .select(AccessTokenRaw::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        if let Some(data) = raw.token {
            Ok(Some(ApiKey::new(data)))
        } else {
            Ok(None)
        }
    }

    pub fn account(
        &'a mut self,
        id: AccountIdInternal,
    ) -> Result<Account, DieselDatabaseError> {
        use crate::schema::account::dsl::*;

        let raw = account
            .filter(account_id.eq(id.as_db_id()))
            .select(AccountRaw::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        serde_json::from_str(raw.json_text.as_str())
            .into_db_error(DieselDatabaseError::Execute, id)
    }

    pub fn account_setup(
        &'a mut self,
        id: AccountIdInternal,
    ) -> Result<AccountSetup, DieselDatabaseError> {
        use crate::schema::account_setup::dsl::*;

        account_setup
            .filter(account_id.eq(id.as_db_id()))
            .select(AccountSetup::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)
    }
}
