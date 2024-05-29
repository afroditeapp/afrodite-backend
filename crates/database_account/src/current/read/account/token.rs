use database::define_current_read_commands;
use diesel::prelude::*;
use error_stack::Result;
use model::{AccessToken, AccessTokenRaw, AccountIdInternal, RefreshToken, RefreshTokenRaw};
use database::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountToken, CurrentSyncReadAccountToken);

impl<C: ConnectionProvider> CurrentSyncReadAccountToken<C> {
    pub fn refresh_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<RefreshToken>, DieselDatabaseError> {
        use crate::schema::refresh_token::dsl::*;

        let raw = refresh_token
            .filter(account_id.eq(id.as_db_id()))
            .select(RefreshTokenRaw::as_select())
            .first(self.conn())
            .into_db_error(id)?;

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
            .into_db_error(id)?;

        if let Some(data) = raw.token {
            Ok(Some(AccessToken::new(data)))
        } else {
            Ok(None)
        }
    }
}
