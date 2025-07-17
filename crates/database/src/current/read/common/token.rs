use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccessToken, AccessTokenRaw, AccessTokenUnixTime, AccountIdInternal, RefreshToken,
    RefreshTokenRaw,
};

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentReadAccountToken);

impl CurrentReadAccountToken<'_> {
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
    ) -> Result<(Option<AccessToken>, Option<AccessTokenUnixTime>), DieselDatabaseError> {
        use crate::schema::access_token::dsl::*;

        let (raw, time) = access_token
            .filter(account_id.eq(id.as_db_id()))
            .select((AccessTokenRaw::as_select(), token_unix_time))
            .first(self.conn())
            .into_db_error(id)?;

        if let Some(data) = raw.token {
            Ok((Some(AccessToken::new(data)), time))
        } else {
            Ok((None, None))
        }
    }
}
