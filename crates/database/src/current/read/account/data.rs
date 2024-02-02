use diesel::prelude::*;
use error_stack::Result;
use futures::Stream;
use model::{
    AccessToken, AccessTokenRaw, Account, AccountData, AccountId, AccountIdDb, AccountIdInternal,
    AccountInternal, AccountSetup, Capabilities, GoogleAccountId, RefreshToken, RefreshTokenRaw,
    SignInWithInfo, SignInWithInfoRaw,
};
use simple_backend_database::{
    diesel_db::{ConnectionProvider, DieselDatabaseError},
    sqlx_db::SqliteDatabaseError,
};
use tokio_stream::StreamExt;

use crate::IntoDatabaseError;

define_read_commands!(CurrentReadAccountData, CurrentSyncReadAccountData);


impl CurrentReadAccountData<'_> {
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

impl<C: ConnectionProvider> CurrentSyncReadAccountData<C> {
    pub fn account(&mut self, id: AccountIdInternal) -> Result<Account, DieselDatabaseError> {
        use crate::schema::account_capabilities;

        let shared_state = self.cmds().common().shared_state(id)?;

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
