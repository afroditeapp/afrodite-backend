use database::{define_current_write_commands, ConnectionProvider, DieselDatabaseError};
use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{
    AccountIdInternal, ChatStateRaw, MatchesSyncVersion, PublicKeyId, ReceivedBlocksSyncVersion, ReceivedLikesSyncVersion, SentBlocksSyncVersion, SentLikesSyncVersion, SetPublicKey, SyncVersionUtils
};
use simple_backend_utils::ContextExt;

use crate::IntoDatabaseError;

mod interaction;
mod message;
mod push_notifications;

define_current_write_commands!(CurrentWriteChat, CurrentSyncWriteChat);

impl<C: ConnectionProvider> CurrentSyncWriteChat<C> {
    pub fn interaction(self) -> interaction::CurrentSyncWriteChatInteraction<C> {
        interaction::CurrentSyncWriteChatInteraction::new(self.cmds)
    }

    pub fn message(self) -> message::CurrentSyncWriteChatMessage<C> {
        message::CurrentSyncWriteChatMessage::new(self.cmds)
    }

    pub fn push_notifications(
        self,
    ) -> push_notifications::CurrentSyncWriteChatPushNotifications<C> {
        push_notifications::CurrentSyncWriteChatPushNotifications::new(self.cmds)
    }

    pub fn insert_chat_state(&mut self, id: AccountIdInternal) -> Result<(), DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        insert_into(chat_state)
            .values((account_id.eq(id.as_db_id()),))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn modify_chat_state(
        &mut self,
        id: AccountIdInternal,
        action: impl Fn(&mut ChatStateRaw),
    ) -> Result<ChatStateChanges, DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        let current = self.read().chat().chat_state(id)?;
        let mut new: ChatStateRaw = current.clone();
        action(&mut new);
        update(chat_state.find(id.as_db_id()))
            .set(&new)
            .execute(self.conn())
            .into_db_error(id)?;

        // Calculate changes
        let changes = ChatStateChanges {
            id,
            received_blocks_sync_version: current
                .received_blocks_sync_version
                .return_new_if_different(new.received_blocks_sync_version),
            received_likes_sync_version: current
                .received_likes_sync_version
                .return_new_if_different(new.received_likes_sync_version),
            sent_likes_sync_version: current
                .sent_likes_sync_version
                .return_new_if_different(new.sent_likes_sync_version),
            sent_blocks_sync_version: current
                .sent_blocks_sync_version
                .return_new_if_different(new.sent_blocks_sync_version),
            matches_sync_version: current
                .matches_sync_version
                .return_new_if_different(new.matches_sync_version),
        };

        Ok(changes)
    }

    pub fn set_public_key(
        &mut self,
        id: AccountIdInternal,
        new_key: SetPublicKey,
    ) -> Result<PublicKeyId, DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        let current = self.read().chat().public_key(id)?;
        let new_id = if let Some(current) = current {
            if current.id.id == i64::MAX {
                return Err(DieselDatabaseError::NoAvailableIds.report());
            } else {
                PublicKeyId {
                    id: current.id.id + 1,
                }
            }
        } else {
            PublicKeyId {
                id: 0,
            }
        };

        insert_into(chat_state)
            .values((
                public_key_id.eq(new_id),
                public_key_version.eq(new_key.version),
                public_key_data.eq(new_key.data),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(new_id)
    }
}

pub struct ChatStateChanges {
    pub id: AccountIdInternal,
    pub received_blocks_sync_version: Option<ReceivedBlocksSyncVersion>,
    pub received_likes_sync_version: Option<ReceivedLikesSyncVersion>,
    pub sent_likes_sync_version: Option<SentLikesSyncVersion>,
    pub sent_blocks_sync_version: Option<SentBlocksSyncVersion>,
    pub matches_sync_version: Option<MatchesSyncVersion>,
}
