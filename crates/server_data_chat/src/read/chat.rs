use database_chat::current::read::GetDbReadCommandsChat;
use model_chat::{
    AccountId, AccountIdInternal, AccountInteractionInternal, AccountInteractionState,
    AllMatchesPage, ChatStateRaw, MatchId, MessageNumber, PageItemCountForNewLikes,
    ReceivedBlocksPage, ReceivedLikeId,
    SentBlocksPage, SentLikesPage, SentMessageId,
};
use server_data::{
    cache::{
        db_iterator::{new_count::DbIteratorStateNewCount, DbIteratorState},
        CacheReadCommon,
    },
    define_cmd_wrapper_read,
    read::DbRead,
    result::Result,
    DataError, IntoDataError,
};

mod public_key;
mod notification;

define_cmd_wrapper_read!(ReadCommandsChat);

impl<'a> ReadCommandsChat<'a> {
    pub fn public_key(self) -> public_key::ReadCommandsChatPublicKey<'a> {
        public_key::ReadCommandsChatPublicKey::new(self.0)
    }
    pub fn notification(self) -> notification::ReadCommandsChatNotification<'a> {
        notification::ReadCommandsChatNotification::new(self.0)
    }
}

impl ReadCommandsChat<'_> {
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
        state: DbIteratorStateNewCount<ReceivedLikeId>,
    ) -> Result<(Vec<AccountId>, PageItemCountForNewLikes), DataError> {
        self.db_read(move |mut cmds| {
            let value = cmds
                .chat()
                .interaction()
                .paged_received_likes_from_received_like_id(
                    id,
                    state.id_at_reset(),
                    state.page().try_into().unwrap_or(i64::MAX),
                    state.previous_id_at_reset(),
                )?;
            Ok(value)
        })
        .await
        .into_error()
    }

    pub async fn matches_page(
        &self,
        id: AccountIdInternal,
        state: DbIteratorState<MatchId>,
    ) -> Result<Vec<AccountId>, DataError> {
        self.db_read(move |mut cmds| {
            let value = cmds.chat().interaction().paged_matches(
                id,
                state.id_at_reset(),
                state.page().try_into().unwrap_or(i64::MAX),
            )?;
            Ok(value)
        })
        .await
        .into_error()
    }

    pub async fn all_sent_blocks(
        &self,
        id: AccountIdInternal,
    ) -> Result<SentBlocksPage, DataError> {
        self.db_read(move |mut cmds| {
            let profiles = cmds.chat().interaction().all_sent_blocks(id)?;
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
            let version = cmds.chat().chat_state(id)?.received_blocks_sync_version;
            Ok(ReceivedBlocksPage {
                profiles: vec![],
                version,
            })
        })
        .await
        .into_error()
    }

    pub async fn all_matches(&self, id: AccountIdInternal) -> Result<AllMatchesPage, DataError> {
        // TODO: Is single SQL query possible? Update: yes, check iterator
        //       implementation.
        // TODO: Remove because match iterator code is enough?

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

        Ok(AllMatchesPage {
            profiles: sent,
            version,
        })
    }

    pub async fn all_pending_messages(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<Vec<u8>>, DataError> {
        self.db_read(move |mut cmds| cmds.chat().message().all_pending_messages(id))
            .await
            .into_error()
    }

    pub async fn all_pending_message_sender_account_ids(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<AccountId>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .message()
                .all_pending_message_sender_account_ids(id)
        })
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
            .map(|interaction| {
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

    pub async fn account_interaction(
        &self,
        account0: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> Result<Option<AccountInteractionInternal>, DataError> {
        let interaction = self
            .db_read(move |mut cmds| {
                cmds.chat()
                    .interaction()
                    .account_interaction(account0, account1)
            })
            .await?;
        Ok(interaction)
    }

    pub async fn unlimited_likes_are_enabled_for_both(
        &self,
        account0: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> Result<bool, DataError> {
        let unlimited_likes_a0 = self
            .read_cache_common(account0, |entry| {
                Ok(entry.other_shared_state.unlimited_likes)
            })
            .await?;

        let unlimited_likes_a1 = self
            .read_cache_common(account1, |entry| {
                Ok(entry.other_shared_state.unlimited_likes)
            })
            .await?;

        Ok(unlimited_likes_a0 == unlimited_likes_a1)
    }
}
