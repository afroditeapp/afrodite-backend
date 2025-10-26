use diesel::{delete, insert_into, prelude::*, upsert::excluded};
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, LoginSession};
use simple_backend_utils::db::MyRunQueryDsl;

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_write_commands};

define_current_write_commands!(CurrentWriteAccountToken);

impl CurrentWriteAccountToken<'_> {
    pub fn login_session(
        mut self,
        id: AccountIdInternal,
        data: Option<LoginSession>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::login_session::dsl::*;

        if let Some(data) = data {
            let access_token_value = data
                .access_token
                .bytes()
                .change_context(DieselDatabaseError::DataFormatConversion)?;
            let refresh_token_value = data
                .refresh_token
                .bytes()
                .change_context(DieselDatabaseError::DataFormatConversion)?;
            insert_into(login_session)
                .values((
                    account_id.eq(id.as_db_id()),
                    access_token.eq(access_token_value),
                    access_token_unix_time.eq(data.access_token_unix_time),
                    access_token_ip_address.eq(data.access_token_ip_address),
                    refresh_token.eq(refresh_token_value),
                ))
                .on_conflict(account_id)
                .do_update()
                .set((
                    access_token.eq(excluded(access_token)),
                    access_token_unix_time.eq(excluded(access_token_unix_time)),
                    access_token_ip_address.eq(excluded(access_token_ip_address)),
                    refresh_token.eq(excluded(refresh_token)),
                ))
                .execute_my_conn(self.conn())
                .into_db_error(id)?;
        } else {
            delete(login_session)
                .filter(account_id.eq(id.as_db_id()))
                .execute(self.conn())
                .into_db_error(id)?;
        }

        Ok(())
    }
}
