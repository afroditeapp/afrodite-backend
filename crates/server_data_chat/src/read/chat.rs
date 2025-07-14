use database_chat::current::read::GetDbReadCommandsChat;
use model_chat::{
    AccountId, AccountIdInternal, AccountInteractionInternal, AccountInteractionState,
    AllMatchesPage, ChatProfileLink, ChatStateRaw, GetSentMessage, MatchId,
    PageItemCountForNewLikes, ReceivedLikeId, SentBlocksPage, SentLikesPage, SentMessageId,
};
use server_data::{
    DataError, IntoDataError,
    cache::{
        CacheReadCommon,
        db_iterator::{DbIteratorState, new_count::DbIteratorStateNewCount},
    },
    db_manager::InternalReading,
    define_cmd_wrapper_read,
    read::DbRead,
    result::Result,
};

mod limits;
mod notification;
mod public_key;

define_cmd_wrapper_read!(ReadCommandsChat);

impl<'a> ReadCommandsChat<'a> {
    pub fn public_key(self) -> public_key::ReadCommandsChatPublicKey<'a> {
        public_key::ReadCommandsChatPublicKey::new(self.0)
    }
    pub fn notification(self) -> notification::ReadCommandsChatNotification<'a> {
        notification::ReadCommandsChatNotification::new(self.0)
    }
    pub fn limits(self) -> limits::ReadCommandsChatLimits<'a> {
        limits::ReadCommandsChatLimits::new(self.0)
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
            Ok(SentLikesPage { profiles })
        })
        .await
        .into_error()
    }

    pub async fn received_likes_page(
        &self,
        id: AccountIdInternal,
        state: DbIteratorStateNewCount<ReceivedLikeId>,
    ) -> Result<(Vec<ChatProfileLink>, PageItemCountForNewLikes), DataError> {
        let (accounts, item_count) = self
            .db_read(move |mut cmds| {
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
            .await?;

        Ok((self.to_chat_profile_links(accounts).await?, item_count))
    }

    pub async fn matches_page(
        &self,
        id: AccountIdInternal,
        state: DbIteratorState<MatchId>,
    ) -> Result<Vec<ChatProfileLink>, DataError> {
        let accounts = self
            .db_read(move |mut cmds| {
                let value = cmds.chat().interaction().paged_matches(
                    id,
                    state.id_at_reset(),
                    state.page().try_into().unwrap_or(i64::MAX),
                )?;
                Ok(value)
            })
            .await?;

        self.to_chat_profile_links(accounts).await
    }

    async fn to_chat_profile_links(
        &self,
        accounts: Vec<AccountId>,
    ) -> Result<Vec<ChatProfileLink>, DataError> {
        let mut links = vec![];
        for id in accounts {
            let x = self
                .cache()
                .read_cache(id, |e| {
                    let version = e
                        .profile
                        .as_ref()
                        .map(|v| v.profile_internal().version_uuid);
                    let content_version = e.media.as_ref().map(|v| v.profile_content_version);
                    let last_seen_time =
                        e.profile.as_ref().and_then(|v| v.last_seen_time(&e.common));
                    Ok(ChatProfileLink::new(
                        id,
                        version,
                        content_version,
                        last_seen_time,
                    ))
                })
                .await?;
            links.push(x);
        }
        Ok(links)
    }

    pub async fn all_sent_blocks(
        &self,
        id: AccountIdInternal,
    ) -> Result<SentBlocksPage, DataError> {
        self.db_read(move |mut cmds| {
            let profiles = cmds.chat().interaction().all_sent_blocks(id)?;
            Ok(SentBlocksPage { profiles })
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

    pub async fn all_sent_messages(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<SentMessageId>, DataError> {
        self.db_read(move |mut cmds| cmds.chat().message().all_sent_messages(id))
            .await
            .into_error()
    }

    pub async fn get_sent_message(
        &self,
        id: AccountIdInternal,
        message: SentMessageId,
    ) -> Result<GetSentMessage, DataError> {
        self.db_read(move |mut cmds| cmds.chat().message().get_sent_message(id, message))
            .await
            .into_error()
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

    pub async fn is_unlimited_likes_enabled(
        &self,
        account: AccountIdInternal,
    ) -> Result<bool, DataError> {
        let unlimited_likes = self
            .read_cache_common(account, |entry| {
                Ok(entry.other_shared_state.unlimited_likes)
            })
            .await?;
        Ok(unlimited_likes)
    }
}
