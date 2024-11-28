use database::{define_current_write_commands, ConnectionProvider, DieselDatabaseError};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::AccountIdInternal;
use model_account::AccountEmailSendingStateRaw;

use crate::{current::read::GetDbReadCommandsAccount, IntoDatabaseError};

define_current_write_commands!(CurrentWriteAccountEmail, CurrentSyncWriteAccountEmail);

impl<C: ConnectionProvider> CurrentSyncWriteAccountEmail<C> {
    pub fn modify_email_sending_states(
        &mut self,
        id: AccountIdInternal,
        mut action: impl FnMut(&mut AccountEmailSendingStateRaw)
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_email_sending_state::dsl::*;

        let mut current_states = self.read().account().email().email_sending_states(id)?;
        action(&mut current_states);

        let current_states_cloned = current_states.clone();
        insert_into(account_email_sending_state)
            .values((
                account_id.eq(id.as_db_id()),
                current_states_cloned,
            ))
            .on_conflict(account_id)
            .do_update()
            .set(current_states)
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
