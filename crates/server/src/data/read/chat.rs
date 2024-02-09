
use crate::{data::IntoDataError, result::Result};
use model::{
    AccountIdInternal, AccountInteractionState, MatchesPage, MessageNumber, PendingMessagesPage,
    ReceivedBlocksPage, ReceivedLikesPage, SentBlocksPage, SentLikesPage,
};

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir},
    ReadCommands,
};
use crate::data::DataError;

define_read_commands!(ReadCommandsChat);

impl ReadCommandsChat<'_> {
    pub async fn all_sent_likes(&self, id: AccountIdInternal) -> Result<SentLikesPage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat().interaction().all_sender_account_interactions(
                id,
                AccountInteractionState::Like,
                true,
            )
        })
        .await
        .map(|profiles| SentLikesPage { profiles })
        .into_error()
    }

    pub async fn all_received_likes(
        &self,
        id: AccountIdInternal,
    ) -> Result<ReceivedLikesPage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .interaction()
                .all_receiver_account_interactions(id, AccountInteractionState::Like)
        })
        .await
        .map(|profiles| ReceivedLikesPage { profiles })
        .into_error()
    }

    pub async fn all_sent_blocks(
        &self,
        id: AccountIdInternal,
    ) -> Result<SentBlocksPage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat().interaction().all_sender_account_interactions(
                id,
                AccountInteractionState::Block,
                false,
            )
        })
        .await
        .map(|profiles| SentBlocksPage { profiles })
        .into_error()
    }

    pub async fn all_received_blocks(
        &self,
        id: AccountIdInternal,
    ) -> Result<ReceivedBlocksPage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .interaction()
                .all_receiver_account_interactions(id, AccountInteractionState::Block)
        })
        .await
        .map(|profiles| ReceivedBlocksPage { profiles })
        .into_error()
    }

    pub async fn all_matches(&self, id: AccountIdInternal) -> Result<MatchesPage, DataError> {
        // TODO: Is single SQL query possible?

        let mut sent = self
            .db_read(move |mut cmds| {
                cmds.chat().interaction().all_sender_account_interactions(
                    id,
                    AccountInteractionState::Match,
                    false,
                )
            })
            .await?;

        let mut received = self
            .db_read(move |mut cmds| {
                cmds.chat()
                    .interaction()
                    .all_receiver_account_interactions(id, AccountInteractionState::Match)
            })
            .await?;

        sent.append(&mut received);

        Ok(MatchesPage { profiles: sent })
    }

    pub async fn all_pending_messages(
        &self,
        id: AccountIdInternal,
    ) -> Result<PendingMessagesPage, DataError> {
        self.db_read(move |mut cmds| cmds.chat().message().all_pending_messages(id))
            .await
            .map(|messages| PendingMessagesPage { messages })
            .into_error()
    }

    /// Get message number of message that receiver has viewed the latest
    pub async fn message_number_of_latest_viewed_message(
        &self,
        id_message_sender: AccountIdInternal,
        id_message_receiver: AccountIdInternal,
    ) -> Result<MessageNumber, DataError> {
        let number = self
            .db_read(move |mut cmds| {
                cmds.chat()
                    .interaction()
                    .account_interaction(id_message_sender, id_message_receiver)
            })
            .await?
            .map(|interaction| {
                // Who is sender and receiver in the interaction data depends
                // on who did the first like
                if interaction.account_id_sender == Some(id_message_sender.into_db_id()) {
                    interaction.receiver_latest_viewed_message
                } else {
                    interaction.sender_latest_viewed_message
                }
            })
            .flatten()
            .unwrap_or_default();
        Ok(number)
    }
}
