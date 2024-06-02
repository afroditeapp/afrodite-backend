mod push_notifications;

use model::{
    AccountId, AccountIdInternal, AccountInteractionState, ChatStateRaw, MatchesPage, MessageNumber, PendingMessagesPage, ReceivedBlocksPage, ReceivedLikesPage, SentBlocksPage, SentLikesPage
};
use server_data::{
    define_server_data_read_commands, read::ReadCommandsProvider, result::Result, DataError,
    IntoDataError,
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

    pub async fn all_received_likes(
        &self,
        id: AccountIdInternal,
    ) -> Result<ReceivedLikesPage, DataError> {
        self.db_read(move |mut cmds| {
            let profiles = cmds
                .chat()
                .interaction()
                .all_receiver_account_interactions(id, AccountInteractionState::Like)?;
            let version = cmds.chat().chat_state(id)?.received_likes_sync_version;
            Ok(ReceivedLikesPage { profiles, version })
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
    ) -> Result<PendingMessagesPage, DataError> {
        self.db_read(move |mut cmds| cmds.chat().message().all_pending_messages(id))
            .await
            .map(|messages| PendingMessagesPage { messages })
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
}
