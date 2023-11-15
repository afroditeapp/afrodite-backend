use database::current::write::chat::CurrentSyncWriteChat;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, Location, ProfileLink, ProfileUpdateInternal, AccountInteractionInternal, PendingMessageId, MessageNumber};

use crate::{data::{cache::CacheError, DataError, IntoDataError, index::location::LocationIndexIteratorState}, internal::Data};


define_write_commands!(WriteCommandsChat);

impl WriteCommandsChat<'_> {
    /// Like or match a profile.
    ///
    /// Returns Ok only if the state change happened.
    pub async fn like_or_match_profile(
        &mut self,
        id_like_sender: AccountIdInternal,
        id_like_receiver: AccountIdInternal,
    ) -> Result<(), DataError> {
        let interaction = self.db_write(move |cmds|
            cmds.into_chat().get_or_create_account_interaction(id_like_sender, id_like_receiver)
        )
            .await?;

        let updated = if interaction.is_like() &&
            interaction.account_id_sender == Some(id_like_sender.into_db_id()) &&
            interaction.account_id_receiver == Some(id_like_receiver.into_db_id()) {
            return Err(DataError::AlreadyDone.report());
        } else if interaction.is_like() &&
            interaction.account_id_sender == Some(id_like_receiver.into_db_id()) &&
            interaction.account_id_receiver == Some(id_like_sender.into_db_id()) {
            interaction
                .try_into_match()
                .change_context(DataError::NotAllowed)?
        } else if interaction.is_match() {
            return Err(DataError::AlreadyDone.report());
        } else {
            interaction
                .try_into_like(id_like_sender, id_like_receiver)
                .change_context(DataError::NotAllowed)?
        };

        self.db_write(move |cmds|
            cmds.into_chat().update_account_interaction(updated)
        )
            .await?;

        Ok(())
    }

    /// Delete a like or block.
    ///
    /// Returns Ok only if the state change happened.
    pub async fn delete_like_or_block(
        &mut self,
        id_sender: AccountIdInternal,
        id_receiver: AccountIdInternal,
    ) -> Result<(), DataError> {
        let interaction = self.db_write(move |cmds|
            cmds.into_chat().get_or_create_account_interaction(id_sender, id_receiver)
        )
            .await?;
        if interaction.is_empty() {
            return Err(DataError::AlreadyDone.report());
        }
        if interaction.account_id_sender != Some(id_sender.into_db_id()) {
            return Err(DataError::NotAllowed.report());
        }
        let updated = interaction
            .try_into_empty()
            .change_context(DataError::NotAllowed)?;
        self.db_write(move |cmds|
            cmds.into_chat().update_account_interaction(updated)
        )
            .await?;

        Ok(())
    }

    /// Block a profile.
    ///
    /// Returns Ok only if the state change happened.
    pub async fn block_profile(
        &mut self,
        id_block_sender: AccountIdInternal,
        id_block_receiver: AccountIdInternal,
    ) -> Result<(), DataError> {
        let interaction = self.db_write(move |cmds|
            cmds.into_chat().get_or_create_account_interaction(id_block_sender, id_block_receiver)
        )
            .await?;
        if interaction.is_blocked() {
            return Err(DataError::AlreadyDone.report());
        }
        let updated = interaction
            .try_into_block(id_block_sender, id_block_receiver)
            .change_context(DataError::NotAllowed)?;
        self.db_write(move |cmds|
            cmds.into_chat().update_account_interaction(updated)
        )
            .await?;

        Ok(())
    }

    /// Delete these pending messages which the receiver has received
    pub async fn delete_pending_message_list(
        &mut self,
        message_receiver: AccountIdInternal,
        messages: Vec<PendingMessageId>,
    ) -> Result<(), DataError> {
        self.db_write(move |cmds|
            cmds.into_chat().delete_pending_message_list(message_receiver, messages)
        )
            .await
    }

    /// Update message number which my account has viewed from the sender
    pub async fn update_message_number_of_latest_viewed_message(
        &self,
        id_my_account: AccountIdInternal,
        id_message_sender: AccountIdInternal,
        new_message_number: MessageNumber,
    ) -> Result<(), DataError> {
        let mut interaction = self.db_read(move |mut cmds| {
            cmds.chat().account_interaction(id_my_account, id_message_sender)
        })
        .await?
        .ok_or(DataError::NotFound.report())?;

        // Prevent marking future messages as viewed
        if new_message_number.message_number > interaction.message_counter {
            return Err(DataError::NotAllowed.report());
        }

        // Who is sender and receiver in the interaction data depends
        // on who did the first like
        let modify_number = if interaction.account_id_sender == Some(id_my_account.into_db_id()) {
            interaction
                .sender_latest_viewed_message
                .as_mut()
        } else {
            interaction
                .receiver_latest_viewed_message
                .as_mut()
        };

        if let Some(number) = modify_number {
            *number = new_message_number;
        } else {
            return Err(DataError::NotAllowed.report());
        }

        self.db_write(move |cmds| {
            cmds.into_chat().update_account_interaction(interaction)
        })
        .await?;

        Ok(())
    }

    /// Insert a new pending message if sender and receiver are a match
    pub async fn insert_pending_message_if_match(
        &mut self,
        sender: AccountIdInternal,
        receiver: AccountIdInternal,
        message: String,
    ) -> Result<(), DataError> {
        self.db_write(move |cmds|
            cmds.into_chat().insert_pending_message_if_match(sender, receiver, message)
        )
            .await
    }
}
