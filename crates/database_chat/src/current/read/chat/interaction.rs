use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountId, AccountIdInternal, AccountInteractionInternal, AccountInteractionState,
    ProfileVisibility, SenderMessageId, UnixTime,
};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadChatInteraction, CurrentSyncReadChatInteraction);

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
                .filter(shared_state::profile_visibility_state_number.eq(ProfileVisibility::Public))
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

    /// Return for example all accounts which have liked the id_receiver account
    /// and specific likeing time
    pub fn all_receiver_account_interactions_with_unix_time(
        &mut self,
        id_receiver: AccountIdInternal,
        with_state: AccountInteractionState,
        unix_time: UnixTime,
    ) -> Result<Vec<AccountId>, DieselDatabaseError> {
        use crate::schema::{account_id, account_interaction::dsl::*};

        let value: Vec<AccountId> = account_interaction
            .inner_join(
                account_id::table.on(account_id_sender.assume_not_null().eq(account_id::id)),
            )
            .filter(account_id_sender.is_not_null())
            .filter(account_id_receiver.eq(id_receiver.as_db_id()))
            .filter(state_number.eq(with_state as i64))
            .filter(state_change_unix_time.eq(unix_time))
            .select(account_id::uuid)
            .order((
                id.desc(),
            ))
            .load(self.conn())
            .into_db_error(())?;

        Ok(value)
    }

    /// Interaction ordering goes from recent to older starting
    /// from `unix_time`.
    pub fn paged_receiver_account_interactions_from_unix_time(
        &mut self,
        id_receiver: AccountIdInternal,
        with_state: AccountInteractionState,
        unix_time: UnixTime,
        page: i64,
    ) -> Result<Vec<AccountId>, DieselDatabaseError> {
        use crate::schema::{account_id, account_interaction::dsl::*};

        const PAGE_SIZE: i64 = 25;

        let value: Vec<AccountId> = account_interaction
            .inner_join(
                account_id::table.on(account_id_sender.assume_not_null().eq(account_id::id)),
            )
            .filter(account_id_sender.is_not_null())
            .filter(account_id_receiver.eq(id_receiver.as_db_id()))
            .filter(state_number.eq(with_state as i64))
            .filter(state_change_unix_time.le(unix_time))
            .select(account_id::uuid)
            .order((
                state_change_unix_time.desc(),
                id.desc(),
            ))
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE.saturating_mul(page))
            .load(self.conn())
            .into_db_error(())?;

        Ok(value)
    }

    /// If sender and receiver is not in match state then default
    /// value is returned.
    pub fn current_expected_sender_id(
        &mut self,
        sender: AccountIdInternal,
        receiver: AccountIdInternal,
    ) -> Result<SenderMessageId, DieselDatabaseError> {
        let value = match self.account_interaction(sender, receiver)? {
            None => SenderMessageId::default(),
            Some(interaction) => {
                if interaction.is_match() {
                    interaction.next_expected_message_id(sender.into_db_id())
                        .copied()
                        .unwrap_or_default()
                } else {
                    SenderMessageId::default()
                }
            }
        };

        Ok(value)
    }
}
