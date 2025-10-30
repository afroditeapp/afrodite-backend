use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, UnixTime};
use model_account::AccountEmailSendingStateRaw;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountEmail);

impl CurrentReadAccountEmail<'_> {
    pub fn email_sending_states(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AccountEmailSendingStateRaw, DieselDatabaseError> {
        use crate::schema::account_email_sending_state::dsl::*;

        account_email_sending_state
            .filter(account_id.eq(id.as_db_id()))
            .select(AccountEmailSendingStateRaw::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(id)
            .map(|data| data.unwrap_or_default())
    }

    pub fn find_account_by_email_verification_token(
        &mut self,
        token: Vec<u8>,
    ) -> Result<Option<(AccountIdInternal, UnixTime)>, DieselDatabaseError> {
        use model::schema::{account, account_id};

        let account_data = account::table
            .inner_join(account_id::table)
            .filter(account::email_verification_token.eq(Some(token)))
            .filter(account::email_verification_token_unix_time.is_not_null())
            .select((
                AccountIdInternal::as_select(),
                account::email_verification_token_unix_time.assume_not_null(),
            ))
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(account_data)
    }
}
