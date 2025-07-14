use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model_chat::{
    AccountId, AccountIdInternal, AccountInteractionInternal, AccountInteractionState, MatchId,
    PageItemCountForNewLikes, ReceivedLikeId,
};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadChatInteraction);

impl CurrentReadChatInteraction<'_> {
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

    pub fn all_sent_blocks(
        &mut self,
        id_sender: AccountIdInternal,
    ) -> Result<Vec<AccountId>, DieselDatabaseError> {
        use crate::schema::{account_id, account_interaction::dsl::*};

        let mut first_list: Vec<AccountId> = account_interaction
            .inner_join(
                account_id::table.on(account_id_block_receiver
                    .assume_not_null()
                    .eq(account_id::id)),
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

        let account_ids_and_received_like_ids: Vec<(AccountId, ReceivedLikeId)> =
            account_interaction
                .inner_join(
                    account_id::table.on(account_id_sender.assume_not_null().eq(account_id::id)),
                )
                .filter(account_id_sender.is_not_null())
                .filter(account_id_receiver.eq(id_receiver.as_db_id()))
                .filter(state_number.eq(AccountInteractionState::Like))
                .filter(received_like_id.is_not_null())
                .filter(received_like_id.le(received_like_id_value))
                .select((account_id::uuid, received_like_id.assume_not_null()))
                .order((received_like_id.desc(),))
                .limit(PAGE_SIZE)
                .offset(PAGE_SIZE.saturating_mul(page))
                .load(self.conn())
                .into_db_error(())?;

        let mut count = 0;
        let account_ids: Vec<AccountId> = if let Some(previous) = received_like_id_previous_value {
            account_ids_and_received_like_ids
                .into_iter()
                .map(|(aid, like_id)| {
                    if like_id.id > previous.id {
                        count += 1;
                    }
                    aid
                })
                .collect()
        } else {
            account_ids_and_received_like_ids
                .into_iter()
                .map(|(aid, _)| aid)
                .collect()
        };

        Ok((account_ids, PageItemCountForNewLikes { c: count }))
    }

    /// Interaction ordering goes from recent to older starting
    /// from `match_id_value`.
    pub fn paged_matches(
        &mut self,
        id_value: AccountIdInternal,
        match_id_value: MatchId,
        page: i64,
    ) -> Result<Vec<AccountId>, DieselDatabaseError> {
        use crate::schema::{account_id, account_interaction::dsl::*};

        const PAGE_SIZE: i64 = 25;

        let account_ids: Vec<AccountId> = account_interaction
            .inner_join(
                account_id::table.on((account_id_sender
                    .assume_not_null()
                    .eq(account_id::id)
                    .and(account_id_receiver.eq(id_value.as_db_id())))
                .or(account_id_receiver
                    .assume_not_null()
                    .eq(account_id::id)
                    .and(account_id_sender.eq(id_value.as_db_id())))),
            )
            .filter(
                (account_id_sender
                    .is_not_null()
                    .and(account_id_receiver.eq(id_value.as_db_id())))
                .or(account_id_receiver
                    .is_not_null()
                    .and(account_id_sender.eq(id_value.as_db_id()))),
            )
            .filter(state_number.eq(AccountInteractionState::Match))
            .filter(match_id.is_not_null())
            .filter(match_id.le(match_id_value))
            .select(account_id::uuid)
            .order((match_id.desc(),))
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE.saturating_mul(page))
            .load(self.conn())
            .into_db_error(())?;

        Ok(account_ids)
    }
}
