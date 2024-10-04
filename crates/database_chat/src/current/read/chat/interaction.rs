use std::i64;

use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountId, AccountIdInternal, AccountInteractionInternal, AccountInteractionState, PageItemCountForNewLikes, ProfileVisibility, ReceivedLikeId
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

    pub fn all_sent_blocks(
        &mut self,
        id_sender: AccountIdInternal,
    ) -> Result<Vec<AccountId>, DieselDatabaseError> {
        use crate::schema::{account_id, account_interaction::dsl::*};

        let mut first_list: Vec<AccountId> = account_interaction
            .inner_join(
                account_id::table.on(account_id_block_receiver.assume_not_null().eq(account_id::id)),
            )
            .filter(account_id_block_receiver.is_not_null())
            .filter(account_id_block_sender.eq(id_sender.as_db_id()))
            .select(account_id::uuid)
            .load(self.conn())
            .into_db_error(())?;

        let second_list: Vec<AccountId> = account_interaction
            .inner_join(
                account_id::table.on(account_id_block_sender.assume_not_null().eq(account_id::id)),
            )
            .filter(account_id_block_sender.is_not_null())
            .filter(account_id_block_receiver.eq(id_sender.as_db_id()))
            .filter(two_way_block.eq(true))
            .select(account_id::uuid)
            .load(self.conn())
            .into_db_error(())?;

        first_list.extend(second_list);

        Ok(first_list)
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

    /// Interaction ordering goes from recent to older starting
    /// from `received_like_id_value`.
    pub fn paged_received_likes_from_received_like_id(
        &mut self,
        id_receiver: AccountIdInternal,
        received_like_id_value: ReceivedLikeId,
        page: i64,
        received_like_id_previous_value: Option<ReceivedLikeId>,
    ) -> Result<(Vec<AccountId>, PageItemCountForNewLikes), DieselDatabaseError> {
        use crate::schema::{account_id, account_interaction::dsl::*};

        const PAGE_SIZE: i64 = 25;

        let account_ids_and_received_like_ids: Vec<(AccountId, ReceivedLikeId)> = account_interaction
            .inner_join(
                account_id::table.on(account_id_sender.assume_not_null().eq(account_id::id)),
            )
            .filter(account_id_sender.is_not_null())
            .filter(account_id_receiver.eq(id_receiver.as_db_id()))
            .filter(state_number.eq(AccountInteractionState::Like))
            .filter(received_like_id.is_not_null())
            .filter(received_like_id.le(received_like_id_value))
            .select((account_id::uuid, received_like_id.assume_not_null()))
            .order((
                received_like_id.desc(),
            ))
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE.saturating_mul(page))
            .load(self.conn())
            .into_db_error(())?;

        let mut count = 0;
        let account_ids: Vec<AccountId> = if let Some(previous) = received_like_id_previous_value {
            account_ids_and_received_like_ids.into_iter().map(|(aid, like_id)| {
                if like_id.id > previous.id {
                    count += 1;
                }
                aid
            }).collect()
        } else {
            account_ids_and_received_like_ids.into_iter().map(|(aid, _)| {
                aid
            }).collect()
        };

        Ok((
            account_ids,
            PageItemCountForNewLikes {
                c: count,
            }
        ))
    }
}
