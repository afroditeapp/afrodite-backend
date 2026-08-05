use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{delete, insert_into, prelude::*, update};
use model::{AccountIdInternal, EmailLoginTokenRow, UnixTime};
use model_account::{
    AccountEmailSendingStateRaw, EmailChangeLimits, EmailLoginLimits, EmailVerificationLimits,
};
use simple_backend_utils::{Result, db::MyRunQueryDsl};

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
        new_token: Vec<u8>,
        token_unix_time: UnixTime,
    ) -> Result<(), DieselDatabaseError> {
        {
            use model::schema::account_email_verification_token::dsl::*;

            insert_into(account_email_verification_token)
                .values((account_id.eq(id.as_db_id()), token.eq(&new_token)))
                .on_conflict(account_id)
                .do_update()
                .set(token.eq(&new_token))
                .execute_my_conn(self.conn())
                .into_db_error(id)?;
        }

        {
            use model::schema::account_email_verification_token_time::dsl::*;

            insert_into(account_email_verification_token_time)
                .values((account_id.eq(id.as_db_id()), unix_time.eq(token_unix_time)))
                .on_conflict(account_id)
                .do_update()
                .set(unix_time.eq(token_unix_time))
                .execute_my_conn(self.conn())
                .into_db_error(id)?;
        }

        Ok(())
    }

    /// Clears email verification token by deleting the row.
    /// The unix_time persists for rate limiting.
    pub fn clear_email_verification_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_email_verification_token::dsl::*;

        delete(account_email_verification_token.find(id.as_db_id()))
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
        use model::schema::account_email_change::dsl::*;

        update(account_email_change.find(id.as_db_id()))
            .set(email_change_verified.eq(true))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn clear_email_change_data(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_email_change::dsl::*;

        delete(account_email_change.find(id.as_db_id()))
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
        use model::schema::account_email_change::dsl::*;

        insert_into(account_email_change)
            .values((
                account_id.eq(id.as_db_id()),
                email_change.eq(new_email.clone()),
                email_change_unix_time.eq(current_time),
                email_change_verification_token.eq(verification_token.clone()),
                email_change_verified.eq(false),
            ))
            .on_conflict(account_id)
            .do_update()
            .set((
                email_change.eq(new_email),
                email_change_unix_time.eq(current_time),
                email_change_verification_token.eq(verification_token),
                email_change_verified.eq(false),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn complete_email_change(
        &mut self,
        id: AccountIdInternal,
        new_email: String,
    ) -> Result<(), DieselDatabaseError> {
        {
            use model::schema::account_email_address_state::dsl::*;

            update(account_email_address_state.find(id.as_db_id()))
                .set(email.eq(Some(new_email)))
                .execute(self.conn())
                .into_db_error(id)?;
        }

        {
            use model::schema::account_email_change::dsl::*;

            delete(account_email_change.find(id.as_db_id()))
                .execute(self.conn())
                .into_db_error(id)?;
        }

        Ok(())
    }

    pub fn set_email_login_enabled(
        &mut self,
        id: AccountIdInternal,
        enabled: bool,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_email_address_state::dsl::*;

        update(account_email_address_state.find(id.as_db_id()))
            .set(email_login_enabled.eq(enabled))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    /// Replace all email login tokens in a single transaction.
    /// Clears existing rows then inserts new ones.
    /// Skips tokens for accounts that no longer exist (deleted accounts).
    pub fn replace_all_email_login_tokens(
        &mut self,
        tokens: Vec<EmailLoginTokenRow>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::{account_email_login_token::dsl as t, account_id::dsl as aid};

        // Clear existing tokens
        delete(t::account_email_login_token)
            .execute(self.conn())
            .into_db_error(())?;

        // Insert new tokens, skipping deleted accounts
        for token in tokens {
            let account_exists: Option<i64> = aid::account_id
                .find(token.account_id.as_db_id())
                .select(aid::id)
                .first(self.conn())
                .optional()
                .into_db_error(())?;

            if account_exists.is_none() {
                continue;
            }

            insert_into(t::account_email_login_token)
                .values((
                    t::account_id.eq(token.account_id.as_db_id()),
                    t::client_token.eq(&token.client_token),
                    t::email_token.eq(&token.email_token),
                    t::unix_time.eq(token.unix_time),
                ))
                .execute_my_conn(self.conn())
                .into_db_error(token.account_id)?;
        }

        Ok(())
    }

    pub fn upsert_email_login_limits(
        &mut self,
        id: AccountIdInternal,
        limits: EmailLoginLimits,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_email_login_limits::dsl::*;

        insert_into(account_email_login_limits)
            .values((account_id.eq(id.as_db_id()), limits.clone()))
            .on_conflict(account_id)
            .do_update()
            .set((
                token_sent_unix_time.eq(limits.token_sent_unix_time),
                monthly_email_count.eq(limits.monthly_email_count),
                monthly_limit_reset_unix_time.eq(limits.monthly_limit_reset_unix_time),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn upsert_email_change_limits(
        &mut self,
        id: AccountIdInternal,
        limits: EmailChangeLimits,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_email_change_limits::dsl::*;

        insert_into(account_email_change_limits)
            .values((account_id.eq(id.as_db_id()), limits.clone()))
            .on_conflict(account_id)
            .do_update()
            .set((
                monthly_email_count.eq(limits.monthly_email_count),
                monthly_limit_reset_unix_time.eq(limits.monthly_limit_reset_unix_time),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn upsert_email_verification_limits(
        &mut self,
        id: AccountIdInternal,
        limits: EmailVerificationLimits,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_email_verification_limits::dsl::*;

        insert_into(account_email_verification_limits)
            .values((account_id.eq(id.as_db_id()), limits.clone()))
            .on_conflict(account_id)
            .do_update()
            .set((
                monthly_email_count.eq(limits.monthly_email_count),
                monthly_limit_reset_unix_time.eq(limits.monthly_limit_reset_unix_time),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn upsert_email_registration_limits(
        &mut self,
        limits: model::EmailRegistrationLimits,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::email_registration_limits::dsl::*;

        insert_into(email_registration_limits)
            .values((
                row_type.eq(0),
                daily_email_count.eq(limits.daily_email_count),
                daily_limit_reset_unix_time.eq(limits.daily_limit_reset_unix_time),
            ))
            .on_conflict(row_type)
            .do_update()
            .set((
                daily_email_count.eq(limits.daily_email_count),
                daily_limit_reset_unix_time.eq(limits.daily_limit_reset_unix_time),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
