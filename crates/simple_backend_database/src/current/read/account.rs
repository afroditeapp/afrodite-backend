use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use futures::Stream;
use model::{
    AccessToken, AccessTokenRaw, Account, AccountId, AccountIdDb, AccountIdInternal, AccountInternal,
    AccountSetup, GoogleAccountId, RefreshToken, RefreshTokenRaw, SignInWithInfo,
    SignInWithInfoRaw, AccountState, Capabilities, AccountData,
};
use tokio_stream::StreamExt;

use crate::{
    diesel::{ConnectionProvider, DieselDatabaseError},
    sqlite::SqliteDatabaseError,
    IntoDatabaseError,
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
                    let account_id = AccountId::new(data.account_id);
                    AccountIdInternal::new(id, account_id)
                })
                .into_db_error(SqliteDatabaseError::Fetch, ())
        })
    }
}

impl<C: ConnectionProvider> CurrentSyncReadAccount<C> {
    pub fn google_account_id_to_account_id(
        &mut self,
        google_id: GoogleAccountId,
    ) -> Result<AccountIdInternal, DieselDatabaseError> {
        use crate::schema::{account_id, sign_in_with_info};

        sign_in_with_info::table
            .inner_join(account_id::table)
            .filter(sign_in_with_info::google_account_id.eq(google_id.as_str()))
            .select(AccountIdInternal::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, google_id)
    }

    pub fn sign_in_with_info(
        &mut self,
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
        &mut self,
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
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<AccessToken>, DieselDatabaseError> {
        use crate::schema::access_token::dsl::*;

        let raw = access_token
            .filter(account_id.eq(id.as_db_id()))
            .select(AccessTokenRaw::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        if let Some(data) = raw.token {
            Ok(Some(AccessToken::new(data)))
        } else {
            Ok(None)
        }
    }

    pub fn account(&mut self, id: AccountIdInternal) -> Result<Account, DieselDatabaseError> {
        use crate::schema::account_capabilities;

        let shared_state = self.conn().read().common().shared_state(id)?;

        let capabilities: Capabilities = account_capabilities::table
            .filter(account_capabilities::account_id.eq(id.as_db_id()))
            .select(Capabilities::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(Account::new_from(shared_state.account_state, capabilities))
    }

    pub fn account_setup(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AccountSetup, DieselDatabaseError> {
        use crate::schema::account_setup::dsl::*;

        account_setup
            .filter(account_id.eq(id.as_db_id()))
            .select(AccountSetup::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)
    }

    pub fn account_data(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AccountData, DieselDatabaseError> {
        use crate::schema::account::dsl::*;

        let account_internal = account
            .filter(account_id.eq(id.as_db_id()))
            .select(AccountInternal::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(AccountData {
            email: account_internal.email,
        })
    }
}
