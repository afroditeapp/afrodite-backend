use diesel::prelude::*;
use error_stack::Result;
use model::{AccessToken, AccountIdInternal, LoginSession, RefreshToken};

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentReadAccountToken);

impl CurrentReadAccountToken<'_> {
    pub fn login_session(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<LoginSession>, DieselDatabaseError> {
        use crate::schema::login_session::dsl::*;

        let data = login_session
            .filter(account_id.eq(id.as_db_id()))
            .select((
                access_token,
                access_token_unix_time,
                access_token_ip_address,
                refresh_token,
            ))
            .first::<(_, _, _, Vec<u8>)>(self.conn())
            .optional()
            .into_db_error(id)?
            .map(|(access, access_time, access_ip, refresh)| LoginSession {
                access_token: AccessToken::new(access),
                access_token_unix_time: access_time,
                access_token_ip_address: access_ip,
                refresh_token: RefreshToken::from_bytes(&refresh),
            });

        Ok(data)
    }
}
