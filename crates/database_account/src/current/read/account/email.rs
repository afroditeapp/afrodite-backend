use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, UnixTime};
use model_account::{AccountEmailSendingStateRaw, EmailAddress};

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
        use model::schema::{account_email_address_state, account_id};

        let data = account_email_address_state::table
            .inner_join(account_id::table)
            .filter(account_email_address_state::email_verification_token.eq(Some(token)))
            .filter(account_email_address_state::email_verification_token_unix_time.is_not_null())
            .select((
                AccountIdInternal::as_select(),
                account_email_address_state::email_verification_token_unix_time.assume_not_null(),
            ))
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(data)
    }

    pub fn find_account_by_email_change_verification_token(
        &mut self,
        token: Vec<u8>,
    ) -> Result<Option<(AccountIdInternal, UnixTime)>, DieselDatabaseError> {
        use model::schema::{account_email_address_state, account_id};

        let data = account_email_address_state::table
            .inner_join(account_id::table)
            .filter(account_email_address_state::email_change_verification_token.eq(Some(token)))
            .filter(account_email_address_state::email_change_unix_time.is_not_null())
            .select((
                AccountIdInternal::as_select(),
                account_email_address_state::email_change_unix_time.assume_not_null(),
            ))
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(data)
    }

    pub fn find_account_by_email_login_token(
        &mut self,
        client_token: Vec<u8>,
        email_token: Vec<u8>,
    ) -> Result<Option<(AccountIdInternal, UnixTime)>, DieselDatabaseError> {
        use model::schema::{account_email_address_state, account_id};

        let data = account_email_address_state::table
            .inner_join(account_id::table)
            .filter(account_email_address_state::email_login_client_token.eq(Some(client_token)))
            .filter(account_email_address_state::email_login_email_token.eq(Some(email_token)))
            .filter(account_email_address_state::email_login_token_unix_time.is_not_null())
            .select((
                AccountIdInternal::as_select(),
                account_email_address_state::email_login_token_unix_time.assume_not_null(),
            ))
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(data)
    }

    pub fn account_id_from_email(
        &mut self,
        email: EmailAddress,
    ) -> Result<Option<AccountIdInternal>, DieselDatabaseError> {
        use crate::schema::{account_email_address_state, account_id};

        let found_account: Option<AccountIdInternal> = account_email_address_state::table
            .inner_join(account_id::table)
            .filter(account_email_address_state::email.eq(email))
            .select(AccountIdInternal::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(found_account)
    }
}
