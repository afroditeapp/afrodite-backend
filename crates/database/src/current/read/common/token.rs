use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccessToken, AccessTokenUnixTime, AccountIdInternal, IpAddressInternal, LoginSession,
    RefreshToken,
};

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
                access_token_previous,
                access_token_ip_address,
                access_token_ip_address_previous,
                refresh_token,
            ))
            .first::<(
                Vec<u8>,
                AccessTokenUnixTime,
                Option<Vec<u8>>,
                IpAddressInternal,
                Option<IpAddressInternal>,
                Vec<u8>,
            )>(self.conn())
            .optional()
            .into_db_error(id)?
            .map(
                |(access, access_time, access_previous, access_ip, access_ip_prev, refresh)| {
                    LoginSession {
                        access_token: AccessToken::from_bytes(&access),
                        access_token_unix_time: access_time,
                        access_token_previous: access_previous
                            .as_ref()
                            .map(|bytes| AccessToken::from_bytes(bytes)),
                        access_token_ip_address: access_ip,
                        access_token_ip_address_previous: access_ip_prev,
                        refresh_token: RefreshToken::from_bytes(&refresh),
                    }
                },
            );

        Ok(data)
    }
}
