use database_chat::current::read::GetDbReadCommandsChat;
use model::{ConversationId, MessageId};
use model_chat::{
    AccountId, AccountIdInternal, AccountInteractionInternal, ChatStateRaw, GetSentMessage,
    LatestSeenMessageInfo, MatchesIteratorState, MessageDeliveryInfo, ProfileLink,
    ReceivedLikesIteratorState, ReceivedLikesPage, ReceivedLikesPageItem, SentBlocksPage,
};
use server_data::{
    DataError, IntoDataError, cache::CacheReadCommon, db_manager::InternalReading,
    define_cmd_wrapper_read, read::DbRead, result::Result,
};

mod limits;
mod notification;
mod privacy;
mod public_key;
mod transfer;

define_cmd_wrapper_read!(ReadCommandsChat);

impl<'a> ReadCommandsChat<'a> {
    pub fn public_key(self) -> public_key::ReadCommandsChatPublicKey<'a> {
        public_key::ReadCommandsChatPublicKey::new(self.0)
    }
    pub fn notification(self) -> notification::ReadCommandsChatNotification<'a> {
        notification::ReadCommandsChatNotification::new(self.0)
    }
    pub fn privacy(self) -> privacy::ReadCommandsChatPrivacy<'a> {
        privacy::ReadCommandsChatPrivacy::new(self.0)
    }
    pub fn limits(self) -> limits::ReadCommandsChatLimits<'a> {
        limits::ReadCommandsChatLimits::new(self.0)
    }
    pub fn transfer(self) -> transfer::ReadCommandsChatTransfer<'a> {
        transfer::ReadCommandsChatTransfer::new(self.0)
    }
}

impl ReadCommandsChat<'_> {
    pub async fn chat_state(&self, id: AccountIdInternal) -> Result<ChatStateRaw, DataError> {
        self.db_read(move |mut cmds| cmds.chat().chat_state(id))
            .await
            .into_error()
    }

    pub async fn received_likes_page(
        &self,
        id: AccountIdInternal,
        state: ReceivedLikesIteratorState,
    ) -> Result<ReceivedLikesPage, DataError> {
        let received_likes = self
            .db_read(move |mut cmds| {
                let value = cmds
                    .chat()
                    .interaction()
                    .paged_received_likes_from_received_like_id(
                        id,
                        state.id_at_reset,
                        state.page,
                    )?;
                Ok(value)
            })
            .await?;

        let mut likes = vec![];
        for (account, like_id, viewed) in received_likes {
            likes.push(ReceivedLikesPageItem {
                p: self.to_profile_link(account).await?,
                not_viewed: if !viewed { Some(like_id) } else { None },
            });
        }
        Ok(ReceivedLikesPage { l: likes })
    }

    pub async fn matches_page(
        &self,
        id: AccountIdInternal,
        state: MatchesIteratorState,
    ) -> Result<Vec<ProfileLink>, DataError> {
        let accounts = self
            .db_read(move |mut cmds| {
                let value =
                    cmds.chat()
                        .interaction()
                        .paged_matches(id, state.id_at_reset, state.page)?;
                Ok(value)
            })
            .await?;

        self.to_profile_link_list(accounts).await
    }

    async fn to_profile_link_list(
        &self,
        accounts: Vec<AccountId>,
    ) -> Result<Vec<ProfileLink>, DataError> {
        let mut links = vec![];
        for id in accounts {
            links.push(self.to_profile_link(id).await?);
        }
        Ok(links)
    }

    async fn to_profile_link(&self, id: AccountId) -> Result<ProfileLink, DataError> {
        self.cache()
            .read_cache(id, |e| {
                Ok(ProfileLink::new(
                    id,
                    e.profile.profile_internal().version_uuid,
                    e.media.profile_content_version,
                    e.profile.last_seen_time().last_seen_time_public(),
                ))
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
            Ok(SentBlocksPage { profiles })
        })
        .await
        .into_error()
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
    ) -> Result<Vec<MessageId>, DataError> {
        self.db_read(move |mut cmds| cmds.chat().message().all_sent_messages(id))
            .await
            .into_error()
    }

    pub async fn get_sent_message(
        &self,
        id: AccountIdInternal,
        message: MessageId,
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

    pub async fn is_video_call_url_already_created(
        &self,
        caller: AccountIdInternal,
        other_user: AccountIdInternal,
    ) -> Result<bool, DataError> {
        let interaction = self.account_interaction(caller, other_user).await?;
        match interaction {
            Some(interaction) => {
                if interaction.account_id_sender == Some(caller.into_db_id()) {
                    Ok(interaction.video_call_url_created_sender)
                } else {
                    Ok(interaction.video_call_url_created_receiver)
                }
            }
            None => Ok(false),
        }
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

    pub async fn get_initial_matches_iterator_state(
        &self,
    ) -> Result<MatchesIteratorState, DataError> {
        let latest_used_id = self
            .db_read(|mut cmds| cmds.chat().global_state())
            .await?
            .next_match_id
            .next_id_to_latest_used_id();
        Ok(MatchesIteratorState {
            id_at_reset: latest_used_id,
            page: 0,
        })
    }

    pub async fn has_unreceived_delivery_info(
        &self,
        id: AccountIdInternal,
    ) -> Result<bool, DataError> {
        self.db_read(move |mut cmds| cmds.chat().message().has_unreceived_delivery_info(id))
            .await
            .into_error()
    }

    pub async fn get_all_delivery_info(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<MessageDeliveryInfo>, DataError> {
        self.db_read(move |mut cmds| cmds.chat().message().get_all_delivery_info(id))
            .await
            .into_error()
    }

    pub async fn has_pending_latest_seen_message_deliveries(
        &self,
        id: AccountIdInternal,
    ) -> Result<bool, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .message()
                .has_pending_latest_seen_message_deliveries(id)
        })
        .await
        .into_error()
    }

    pub async fn get_pending_latest_seen_message_deliveries(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<LatestSeenMessageInfo>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .message()
                .get_pending_latest_seen_message_deliveries(id)
        })
        .await
        .into_error()
    }

    pub async fn get_conversation_id(
        &self,
        owner_id: AccountIdInternal,
        other_id: AccountIdInternal,
    ) -> Result<Option<ConversationId>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .message()
                .get_conversation_id(owner_id, other_id)
        })
        .await
        .into_error()
    }
}
