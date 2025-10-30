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

    pub fn clear_email_verification_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        update(account.find(id.as_db_id()))
            .set((
                email_verification_token.eq(None::<Vec<u8>>),
                email_verification_token_unix_time.eq(None::<UnixTime>),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
