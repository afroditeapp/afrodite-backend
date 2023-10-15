use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use futures::Stream;
use model::{
    AccessToken, AccessTokenRaw, Account, AccountId, AccountIdDb, AccountIdInternal, AccountRaw,
    AccountSetup, GoogleAccountId, RefreshToken, RefreshTokenRaw, SignInWithInfo,
    SignInWithInfoRaw, schema::access_token::account_id, AccountInteractionInternal,
};
use tokio_stream::StreamExt;

use crate::{
    diesel::{ConnectionProvider, DieselDatabaseError},
    sqlite::SqliteDatabaseError,
    IntoDatabaseError, current::write::account,
};

define_read_commands!(CurrentReadChat, CurrentSyncReadChat);

impl<C: ConnectionProvider> CurrentSyncReadChat<C> {
    pub fn account_interaction(
        &mut self,
        account2: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> Result<Option<AccountInteractionInternal>, DieselDatabaseError> {
        use crate::schema::{account_interaction_index::dsl::*, account_interaction::dsl::*};

        let interaction_id_value = account_interaction_index
            .filter(account_id_first.eq(account1.as_db_id()))
            .filter(account_id_second.eq(account2.as_db_id()))
            .select(interaction_id)
            .first::<i64>(self.conn())
            .optional()
            .into_db_error(DieselDatabaseError::Execute, (account1, account2))?;

        let interaction_id_value = match interaction_id_value {
            Some(value) => value,
            None => return Ok(None),
        };

        let value = account_interaction
            .filter(id.eq(interaction_id_value))
            .select(AccountInteractionInternal::as_select())
            .first(self.conn())
            .into_db_error(DieselDatabaseError::Execute, (account1, account2))?;

        Ok(Some(value))
    }

    // pub fn sign_in_with_info(
    //     &mut self,
    //     id: AccountIdInternal,
    // ) -> Result<SignInWithInfo, DieselDatabaseError> {
    //     use crate::schema::sign_in_with_info::dsl::*;

    //     sign_in_with_info
    //         .filter(account_id.eq(id.as_db_id()))
    //         .select(SignInWithInfoRaw::as_select())
    //         .first(self.conn())
    //         .into_db_error(DieselDatabaseError::Execute, id)
    //         .map(Into::into)
    // }
}
