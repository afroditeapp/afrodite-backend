use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, UnixTime};
use model_account::AccountEmailSendingStateRaw;
use simple_backend_utils::db::MyRunQueryDsl;

use crate::{IntoDatabaseError, current::read::GetDbReadCommandsAccount};

define_current_write_commands!(CurrentWriteAccountEmail);

impl CurrentWriteAccountEmail<'_> {
    pub fn modify_email_sending_states(
        &mut self,
        id: AccountIdInternal,
        mut action: impl FnMut(&mut AccountEmailSendingStateRaw),
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_email_sending_state::dsl::*;

        let mut current_states = self.read().account().email().email_sending_states(id)?;
        action(&mut current_states);

        let current_states_cloned = current_states.clone();
        insert_into(account_email_sending_state)
            .values((account_id.eq(id.as_db_id()), current_states_cloned))
            .on_conflict(account_id)
            .do_update()
            .set(current_states)
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn set_email_verification_token(
        mut self,
        id: AccountIdInternal,
        token: Vec<u8>,
        token_unix_time: UnixTime,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        update(account.find(id.as_db_id()))
            .set((
                email_verification_token.eq(Some(token)),
                email_verification_token_unix_time.eq(Some(token_unix_time)),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    /// Does not clear email_verification_token_unix_time to limit
    /// verification email sending.
    pub fn clear_email_verification_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        update(account.find(id.as_db_id()))
            .set(email_verification_token.eq(None::<Vec<u8>>))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    /// Does not modify email_change_verification_token, so that email link
    /// will work multiple times.
    pub fn verify_pending_email_address(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        update(account.find(id.as_db_id()))
            .set(email_change_verified.eq(true))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    /// Does not clear email_change_unix_time so that email
    /// changing limit will work.
    pub fn clear_email_change_data(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        update(account.find(id.as_db_id()))
            .set((
                email_change.eq(None::<String>),
                email_change_verification_token.eq(None::<Vec<u8>>),
                email_change_verified.eq(false),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn init_email_change(
        &mut self,
        id: AccountIdInternal,
        new_email: String,
        current_time: UnixTime,
        verification_token: Vec<u8>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        update(account.find(id.as_db_id()))
            .set((
                email_change.eq(Some(new_email)),
                email_change_unix_time.eq(Some(current_time)),
                email_change_verification_token.eq(Some(verification_token)),
                email_change_verified.eq(false),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    /// Does not clear email_change_unix_time so that email
    /// changing limit will work.
    pub fn complete_email_change(
        &mut self,
        id: AccountIdInternal,
        new_email: String,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        update(account.find(id.as_db_id()))
            .set((
                email.eq(Some(new_email)),
                email_change.eq(None::<String>),
                email_change_verification_token.eq(None::<Vec<u8>>),
                email_change_verified.eq(false),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn set_email_login_token(
        &mut self,
        id: AccountIdInternal,
        token: Vec<u8>,
        token_unix_time: UnixTime,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        update(account.find(id.as_db_id()))
            .set((
                email_login_token.eq(Some(token)),
                email_login_token_unix_time.eq(Some(token_unix_time)),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    /// Does not clear email_login_token_unix_time, so that email sending limit
    /// will work.
    pub fn clear_email_login_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        update(account.find(id.as_db_id()))
            .set(email_login_token.eq(None::<Vec<u8>>))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
