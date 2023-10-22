use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use futures::Stream;
use model::{
    AccessToken, AccessTokenRaw, Account, AccountId, AccountIdDb, AccountIdInternal, AccountInternal,
    AccountSetup, GoogleAccountId, RefreshToken, RefreshTokenRaw, SignInWithInfo,
    SignInWithInfoRaw, SharedState, Capabilities,
};
use tokio_stream::StreamExt;

use crate::{
    diesel::{ConnectionProvider, DieselDatabaseError},
    sqlite::SqliteDatabaseError,
    IntoDatabaseError,
};

define_read_commands!(CurrentReadAccount, CurrentSyncReadCommon);


impl<C: ConnectionProvider> CurrentSyncReadCommon<C> {
    pub fn shared_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<SharedState, DieselDatabaseError> {
        use crate::schema::shared_state::dsl::*;

        shared_state
            .filter(account_id.eq(id.as_db_id()))
            .select(SharedState::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)
    }

    pub fn account_capabilities(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Capabilities, DieselDatabaseError> {
        use crate::schema::account_capabilities::dsl::*;

        account_capabilities
            .filter(account_id.eq(id.as_db_id()))
            .select(Capabilities::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)
    }
}
