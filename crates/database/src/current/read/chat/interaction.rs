use diesel::prelude::*;
use error_stack::Result;
use model::{AccountId, AccountIdInternal, AccountInteractionInternal, AccountInteractionState};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};
use tokio_stream::StreamExt;

use crate::IntoDatabaseError;

define_read_commands!(CurrentReadChatInteraction, CurrentSyncReadChatInteraction);

impl<C: ConnectionProvider> CurrentSyncReadChatInteraction<C> {
    pub fn account_interaction(
        &mut self,
        account2: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> Result<Option<AccountInteractionInternal>, DieselDatabaseError> {
        use crate::schema::{account_interaction::dsl::*, account_interaction_index::dsl::*};

        let interaction_id_value = account_interaction_index
            .filter(account_id_first.eq(account1.as_db_id()))
            .filter(account_id_second.eq(account2.as_db_id()))
            .select(interaction_id)
            .first::<i64>(self.conn())
            .optional()
            .into_db_error((account1, account2))?;

        let interaction_id_value = match interaction_id_value {
            Some(value) => value,
            None => return Ok(None),
        };

        let value = account_interaction
            .filter(id.eq(interaction_id_value))
            .select(AccountInteractionInternal::as_select())
            .first(self.conn())
            .into_db_error((account1, account2))?;

        Ok(Some(value))
    }

    /// Return for example all accounts which id_sender account has liked
    pub fn all_sender_account_interactions(
        &mut self,
        id_sender: AccountIdInternal,
        with_state: AccountInteractionState,
        only_public_profiles: bool,
    ) -> Result<Vec<AccountId>, DieselDatabaseError> {
        use crate::schema::{account_id, account_interaction::dsl::*, shared_state};

        let partial_command = account_interaction
            .inner_join(
                account_id::table.on(account_id_receiver.assume_not_null().eq(account_id::id)),
            )
            .inner_join(
                shared_state::table.on(account_id_receiver
                    .assume_not_null()
                    .eq(shared_state::account_id)),
            )
            .filter(account_id_receiver.is_not_null())
            .filter(account_id_sender.eq(id_sender.as_db_id()))
            .filter(state_number.eq(with_state as i64));

        let value: Vec<AccountId> = if only_public_profiles {
            partial_command
                .filter(shared_state::is_profile_public.eq(true))
                .select(account_id::uuid)
                .load(self.conn())
                .into_db_error(())?
        } else {
            partial_command
                .select(account_id::uuid)
                .load(self.conn())
                .into_db_error(())?
        };

        Ok(value)
    }

    /// Return for example all accounts which have liked the id_receiver account
    pub fn all_receiver_account_interactions(
        &mut self,
        id_receiver: AccountIdInternal,
        with_state: AccountInteractionState,
    ) -> Result<Vec<AccountId>, DieselDatabaseError> {
        use crate::schema::{account_id, account_interaction::dsl::*};

        let value: Vec<AccountId> = account_interaction
            .inner_join(
                account_id::table.on(account_id_sender.assume_not_null().eq(account_id::id)),
            )
            .filter(account_id_sender.is_not_null())
            .filter(account_id_receiver.eq(id_receiver.as_db_id()))
            .filter(state_number.eq(with_state as i64))
            .select(account_id::uuid)
            .load(self.conn())
            .into_db_error(())?;

        Ok(value)
    }
}
