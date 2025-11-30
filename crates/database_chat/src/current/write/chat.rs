use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{ConversationId, UnixTime};
use model_chat::{
    AccountIdInternal, CHAT_GLOBAL_STATE_ROW_TYPE, ChatStateRaw, MatchId, NewReceivedLikesCount,
    PublicKeyId, ReceivedLikesSyncVersion, SyncVersionUtils,
};
use simple_backend_utils::{ContextExt, db::MyRunQueryDsl};

use crate::{IntoDatabaseError, current::read::GetDbReadCommandsChat};

mod interaction;
mod limits;
mod message;
mod notification;
mod privacy;
mod report;

define_current_write_commands!(CurrentWriteChat);

impl<'a> CurrentWriteChat<'a> {
    pub fn interaction(self) -> interaction::CurrentWriteChatInteraction<'a> {
        interaction::CurrentWriteChatInteraction::new(self.cmds)
    }

    pub fn message(self) -> message::CurrentWriteChatMessage<'a> {
        message::CurrentWriteChatMessage::new(self.cmds)
    }

    pub fn report(self) -> report::CurrentWriteChatReport<'a> {
        report::CurrentWriteChatReport::new(self.cmds)
    }

    pub fn notification(self) -> notification::CurrentWriteChatNotification<'a> {
        notification::CurrentWriteChatNotification::new(self.cmds)
    }

    pub fn privacy(self) -> privacy::CurrentWriteChatPrivacy<'a> {
        privacy::CurrentWriteChatPrivacy::new(self.cmds)
    }

    pub fn limits(self) -> limits::CurrentWriteChatLimits<'a> {
        limits::CurrentWriteChatLimits::new(self.cmds)
    }
}

impl CurrentWriteChat<'_> {
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
            received_likes_change: current
                .received_likes_sync_version
                .return_new_if_different(new.received_likes_sync_version)
                .map(|changed_sync_version| ReceivedLikesChangeInfo {
                    current_version: changed_sync_version,
                    current_count: new.new_received_likes_count,
                    previous_count: current.new_received_likes_count,
                }),
        };

        Ok(changes)
    }

    pub fn add_public_key(
        &mut self,
        id: AccountIdInternal,
        new_key: Vec<u8>,
    ) -> Result<PublicKeyId, DieselDatabaseError> {
        use model::schema::public_key::dsl::*;

        let current_time = UnixTime::current_time();

        let current = self.read().chat().public_key().latest_public_key_id(id)?;
        let new_id = if let Some(current) = current {
            if current.id == i64::MAX {
                return Err(DieselDatabaseError::NoAvailableIds.report());
            } else {
                PublicKeyId { id: current.id + 1 }
            }
        } else {
            PublicKeyId { id: 0 }
        };

        insert_into(public_key)
            .values((
                account_id.eq(id.as_db_id()),
                key_id.eq(new_id),
                key_data.eq(&new_key),
                key_added_unix_time.eq(current_time),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(new_id)
    }

    /// Return unused MatchId
    pub fn upsert_next_match_id(&mut self) -> Result<MatchId, DieselDatabaseError> {
        use model::schema::chat_global_state::dsl::*;

        let current = self.read().chat().global_state()?.next_match_id;
        let next = current.increment();

        insert_into(chat_global_state)
            .values((
                row_type.eq(CHAT_GLOBAL_STATE_ROW_TYPE),
                next_match_id.eq(next),
            ))
            .on_conflict(row_type)
            .do_update()
            .set(next_match_id.eq(next))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(current)
    }

    /// Return unused ConversationId
    pub fn upsert_next_conversation_id(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<ConversationId, DieselDatabaseError> {
        use model::schema::chat_state::dsl::*;

        let current = self.read().chat().chat_state(id)?.next_conversation_id;
        let next = current.increment();

        update(chat_state)
            .filter(account_id.eq(id.as_db_id()))
            .set(next_conversation_id.eq(next))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(current)
    }
}

pub struct ChatStateChanges {
    pub id: AccountIdInternal,
    pub received_likes_change: Option<ReceivedLikesChangeInfo>,
}

pub struct ReceivedLikesChangeInfo {
    pub current_version: ReceivedLikesSyncVersion,
    pub current_count: NewReceivedLikesCount,
    pub previous_count: NewReceivedLikesCount,
}

pub struct ReceiverBlockedSender;
