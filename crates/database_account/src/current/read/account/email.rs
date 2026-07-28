use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, EmailLoginTokenRow, UnixTime};
use model_account::{AccountEmailSendingStateRaw, EmailAddress, EmailLoginTokens};

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
        use model::schema::{
            account_email_verification_token, account_email_verification_token_time, account_id,
        };

        let data = account_email_verification_token::table
            .inner_join(account_id::table)
            .inner_join(
                account_email_verification_token_time::table
                    .on(account_email_verification_token_time::account_id
                        .eq(account_email_verification_token::account_id)),
            )
            .filter(account_email_verification_token::token.eq(token))
            .select((
                AccountIdInternal::as_select(),
                account_email_verification_token_time::unix_time,
            ))
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(data)
    }

    pub fn email_verification_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(Option<Vec<u8>>, Option<UnixTime>), DieselDatabaseError> {
        use model::schema::{
            account_email_verification_token, account_email_verification_token_time,
        };

        let token = account_email_verification_token::table
            .filter(account_email_verification_token::account_id.eq(id.as_db_id()))
            .select(account_email_verification_token::token)
            .first(self.conn())
            .optional()
            .into_db_error(id)?;

        let time = account_email_verification_token_time::table
            .filter(account_email_verification_token_time::account_id.eq(id.as_db_id()))
            .select(account_email_verification_token_time::unix_time)
            .first(self.conn())
            .optional()
            .into_db_error(id)?;

        Ok((token, time))
    }

    pub fn email_verification_token_time(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<UnixTime>, DieselDatabaseError> {
        use model::schema::account_email_verification_token_time::dsl::*;

        let data = account_email_verification_token_time
            .filter(account_id.eq(id.as_db_id()))
            .select(unix_time)
            .first(self.conn())
            .optional()
            .into_db_error(id)?;

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

    pub fn email_login_tokens(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<EmailLoginTokens, DieselDatabaseError> {
        use model::schema::account_email_login_token::dsl::*;

        let tokens: Option<(Vec<u8>, Vec<u8>)> = account_email_login_token
            .filter(account_id.eq(id.as_db_id()))
            .select((client_token, email_token))
            .first(self.conn())
            .optional()
            .into_db_error(id)?;

        Ok(match tokens {
            Some((c, e)) => model_account::EmailLoginTokens {
                client_token: Some(c),
                email_token: Some(e),
            },
            None => model_account::EmailLoginTokens {
                client_token: None,
                email_token: None,
            },
        })
    }

    pub fn email_login_token_time(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<UnixTime>, DieselDatabaseError> {
        use model::schema::account_email_login_token::dsl::*;

        let time: Option<UnixTime> = account_email_login_token
            .filter(account_id.eq(id.as_db_id()))
            .select(unix_time)
            .first(self.conn())
            .optional()
            .into_db_error(id)?;

        Ok(time)
    }

    /// Load all login tokens from DB at once.
    pub fn all_email_login_tokens(
        &mut self,
    ) -> Result<Vec<EmailLoginTokenRow>, DieselDatabaseError> {
        use model::schema::{account_email_login_token, account_id};

        let data: Vec<EmailLoginTokenRow> = account_email_login_token::table
            .inner_join(account_id::table)
            .filter(account_email_login_token::unix_time.is_not_null())
            .select((
                AccountIdInternal::as_select(),
                account_email_login_token::client_token,
                account_email_login_token::email_token,
                account_email_login_token::unix_time.assume_not_null(),
            ))
            .load(self.conn())
            .into_db_error(())?
            .into_iter()
            .map(
                |(account_id, client_token, email_token, unix_time)| EmailLoginTokenRow {
                    account_id,
                    client_token,
                    email_token,
                    unix_time,
                },
            )
            .collect();

        Ok(data)
    }

    pub fn email_login_token_sent_time(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<UnixTime>, DieselDatabaseError> {
        use model::schema::account_email_login_limits::dsl::*;

        let time: Option<UnixTime> = account_email_login_limits
            .filter(account_id.eq(id.as_db_id()))
            .select(token_sent_unix_time)
            .first(self.conn())
            .optional()
            .into_db_error(id)?;

        Ok(time)
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
