use futures::future::Pending;
use model::{AccountInteractionState, AccountId, AccountIdInternal, SentLikesPage, ReceivedLikesPage, SentBlocksPage, ReceivedBlocksPage, MatchesPage, PendingMessagesPage};
use error_stack::{Result, ResultExt};
use crate::data::DataError;

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir},
    ReadCommands,
};

define_read_commands!(ReadCommandsChat);

impl ReadCommandsChat<'_> {

    pub async fn all_sent_likes(
        &self,
        id: AccountIdInternal,
    ) -> Result<SentLikesPage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .all_sender_account_interactions(
                    id,
                    AccountInteractionState::Like,
                    true,
                )
        })
        .await
        .map(|profiles| SentLikesPage { profiles })
    }

    pub async fn all_received_likes(
        &self,
        id: AccountIdInternal,
    ) -> Result<ReceivedLikesPage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .all_receiver_account_interactions(
                    id,
                    AccountInteractionState::Like,
                )
        })
        .await
        .map(|profiles| ReceivedLikesPage { profiles })
    }

    pub async fn all_sent_blocks(
        &self,
        id: AccountIdInternal,
    ) -> Result<SentBlocksPage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .all_sender_account_interactions(
                    id,
                    AccountInteractionState::Block,
                    false,
                )
        })
        .await
        .map(|profiles| SentBlocksPage { profiles })
    }

    pub async fn all_received_blocks(
        &self,
        id: AccountIdInternal,
    ) -> Result<ReceivedBlocksPage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .all_receiver_account_interactions(
                    id,
                    AccountInteractionState::Block,
                )
        })
        .await
        .map(|profiles| ReceivedBlocksPage { profiles })
    }

    pub async fn all_matches(
        &self,
        id: AccountIdInternal,
    ) -> Result<MatchesPage, DataError> {
        // TODO: Is single SQL query possible?

        let mut sent = self.db_read(move |mut cmds| {
            cmds.chat()
                .all_sender_account_interactions(
                    id,
                    AccountInteractionState::Match,
                    false,
                )
        })
        .await?;

        let mut received = self.db_read(move |mut cmds| {
            cmds.chat()
                .all_receiver_account_interactions(
                    id,
                    AccountInteractionState::Match,
                )
        })
        .await?;

        sent.append(&mut received);

        Ok(MatchesPage {
            profiles: sent,
        })
    }

    pub async fn all_pending_messages(
        &self,
        id: AccountIdInternal,
    ) -> Result<PendingMessagesPage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat().all_pending_messages(id)
        })
        .await
        .map(|messages| PendingMessagesPage { messages })
    }
}
