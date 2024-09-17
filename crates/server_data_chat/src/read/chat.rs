mod push_notifications;

use std::i64;

use model::{
    AccountId, AccountIdInternal, AccountInteractionState, ChatStateRaw, GetPublicKey, MatchesPage, MessageNumber, PendingMessageAndMessageData, PublicKeyVersion, ReceivedBlocksPage, SentBlocksPage, SentLikesPage, SentMessageId
};
use server_data::{
 cache::received_likes::ReceivedLikesIteratorState, define_server_data_read_commands, read::ReadCommandsProvider, result::Result, DataError, IntoDataError
};

use self::push_notifications::ReadCommandsChatPushNotifications;

define_server_data_read_commands!(ReadCommandsChat);
define_db_read_command!(ReadCommandsChat);

impl<C: ReadCommandsProvider> ReadCommandsChat<C> {
    pub fn push_notifications(self) -> ReadCommandsChatPushNotifications<C> {
        ReadCommandsChatPushNotifications::new(self.cmds)
    }
}

impl<C: ReadCommandsProvider> ReadCommandsChat<C> {
    pub async fn chat_state(&self, id: AccountIdInternal) -> Result<ChatStateRaw, DataError> {
        self.db_read(move |mut cmds| cmds.chat().chat_state(id))
            .await
            .into_error()
    }

    pub async fn all_sent_likes(&self, id: AccountIdInternal) -> Result<SentLikesPage, DataError> {
        self.db_read(move |mut cmds| {
            let profiles = cmds.chat().interaction().all_sender_account_interactions(
                id,
                AccountInteractionState::Like,
                true,
            )?;
            let version = cmds.chat().chat_state(id)?.sent_likes_sync_version;
            Ok(SentLikesPage { profiles, version })
        })
        .await
        .into_error()
    }

    pub async fn received_likes_page(
        &self,
        id: AccountIdInternal,
        state: ReceivedLikesIteratorState,
    ) -> Result<Vec<AccountId>, DataError> {
        self.db_read(move |mut cmds| {
            let profiles = match state {
                ReceivedLikesIteratorState::FirstPage { first_like_time } => {
                    let mut profiles = cmds
                        .chat()
                        .interaction()
                        .all_receiver_account_interactions_with_unix_time(id, AccountInteractionState::Like, first_like_time)?;
                    let older_time = first_like_time.decrement();
                    let older_likes = cmds
                        .chat()
                        .interaction()
                        .paged_receiver_account_interactions_from_unix_time(id, AccountInteractionState::Like, older_time, 0)?;
                    profiles.extend(older_likes);
                    profiles
                }
                ReceivedLikesIteratorState::NextPages { time_value, page } =>
                    cmds
                        .chat()
                        .interaction()
                        .paged_receiver_account_interactions_from_unix_time(
                            id,
                            AccountInteractionState::Like,
                            time_value,
                            page.get().try_into().unwrap_or(i64::MAX),
                        )?,
            };

            Ok(profiles)
        })
        .await
        .into_error()
    }

    pub async fn all_sent_blocks(
        &self,
        id: AccountIdInternal,
    ) -> Result<SentBlocksPage, DataError> {
        self.db_read(move |mut cmds| {
            let profiles = cmds.chat().interaction().all_sender_account_interactions(
                id,
                AccountInteractionState::Block,
                false,
            )?;
            let version = cmds.chat().chat_state(id)?.sent_blocks_sync_version;
            Ok(SentBlocksPage { profiles, version })
        })
        .await
        .into_error()
    }

    pub async fn all_received_blocks(
        &self,
        id: AccountIdInternal,
    ) -> Result<ReceivedBlocksPage, DataError> {
        self.db_read(move |mut cmds| {
            let profiles = cmds
                .chat()
                .interaction()
                .all_receiver_account_interactions(id, AccountInteractionState::Block)?;
            let version = cmds.chat().chat_state(id)?.received_blocks_sync_version;
            Ok(ReceivedBlocksPage { profiles, version })
        })
        .await
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

        let version = self
            .db_read(move |mut cmds| Ok(cmds.chat().chat_state(id)?.matches_sync_version))
            .await?;

        Ok(MatchesPage {
            profiles: sent,
            version,
        })
    }

    pub async fn all_pending_messages(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<PendingMessageAndMessageData>, DataError> {
        self.db_read(move |mut cmds| cmds.chat().message().all_pending_messages(id))
            .await
            .into_error()
    }

    pub async fn all_pending_message_sender_account_ids(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<AccountId>, DataError> {
        self.db_read(move |mut cmds| cmds.chat().message().all_pending_message_sender_account_ids(id))
            .await
            .into_error()
    }

    pub async fn all_sent_messages(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<SentMessageId>, DataError> {
        self.db_read(move |mut cmds| cmds.chat().message().all_sent_messages(id))
            .await
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
            .and_then(|interaction| {
                // Who is sender and receiver in the interaction data depends
                // on who did the first like
                if interaction.account_id_sender == Some(id_message_sender.into_db_id()) {
                    interaction.receiver_latest_viewed_message
                } else {
                    interaction.sender_latest_viewed_message
                }
            })
            .unwrap_or_default();
        Ok(number)
    }

    pub async fn is_match(
        &self,
        account0: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> Result<bool, DataError> {
        let is_match = self
            .db_read(move |mut cmds| {
                cmds.chat()
                    .interaction()
                    .account_interaction(account0, account1)
            })
            .await?
            .map(|interaction| {
                interaction.is_match()
            })
            .unwrap_or_default();
        Ok(is_match)
    }

    pub async fn unlimited_likes_are_enabled_for_both(
        &self,
        account0: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> Result<bool, DataError> {
        let unlimited_likes_a0 = self.read_cache(account0, |entry| {
            entry.other_shared_state.unlimited_likes
        }).await?;

        let unlimited_likes_a1 = self.read_cache(account1, |entry| {
            entry.other_shared_state.unlimited_likes
        }).await?;

        Ok(unlimited_likes_a0 == unlimited_likes_a1)
    }

    pub async fn get_public_key(
        &self,
        id: AccountIdInternal,
        version: PublicKeyVersion,
    ) -> Result<GetPublicKey, DataError> {
        self.db_read(move |mut cmds| cmds.chat().public_key(id, version))
            .await
            .map(|key| GetPublicKey { key })
            .into_error()
    }
}
