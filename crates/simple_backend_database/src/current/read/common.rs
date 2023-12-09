use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use futures::Stream;
use model::{
    AccessToken, AccessTokenRaw, Account, AccountId, AccountIdDb, AccountIdInternal, AccountInternal,
    AccountSetup, GoogleAccountId, RefreshToken, RefreshTokenRaw, SignInWithInfo,
    SignInWithInfoRaw, SharedStateInternal, Capabilities, SharedState, AccountState,
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

        let data: SharedStateInternal = shared_state
            .filter(account_id.eq(id.as_db_id()))
            .select(SharedStateInternal::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        let state: AccountState = TryInto::<AccountState>::try_into(
            data.account_state_number
        )
            .change_context(DieselDatabaseError::DataFormatConversion)?;

        Ok(SharedState {
            account_state: state,
            is_profile_public: data.is_profile_public,
        })
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
