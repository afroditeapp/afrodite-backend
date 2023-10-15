
use diesel::{prelude::*, Associations};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    macros::diesel_string_wrapper, AccessToken, AccountIdDb, AccountIdInternal, RefreshToken, AccountId,
};

#[derive(Debug, Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::account_interaction)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountInteractionInternal {
    pub id: i64,
    pub state_number: i64,
    pub account_id_sender: Option<AccountIdDb>,
    pub account_id_receiver: Option<AccountIdDb>,
    pub message_counter: i64,
    pub sender_latest_viewed_message: Option<i64>,
    pub receiver_latest_viewed_message: Option<i64>,
}

#[derive(Debug, Clone, Copy)]
pub enum AccountInteractionState {
    Empty = 0,
    Like = 1,
    Match = 2,
    Block = 3,
}

impl TryFrom<i64> for AccountInteractionState {
    type Error = ();

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Empty),
            1 => Ok(Self::Like),
            2 => Ok(Self::Match),
            3 => Ok(Self::Block),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::pending_messages)]
#[diesel(check_for_backend(crate::Db))]
pub struct PendingMessageInternal {
    pub id: i64,
    pub account_id_sender: AccountIdDb,
    pub account_id_receiver: AccountIdDb,
    pub unix_time: i64,
    pub message_number: i64,
    pub message_text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct SentLikesPage {
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct ReceivedLikesPage {
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct MatchesPage {
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct SentBlocksPage {
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct ReceivedBlocksPage {
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct PendingMessage {
    pub id: PendingMessageId,
    /// Unix time when server received the message.
    pub unix_time: i64,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct PendingMessagesPage {
    pub messages: Vec<PendingMessage>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct PendingMessageId {
    /// Sender of the message.
    pub account_id_sender: AccountId,
    pub message_number: MessageNumber,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct PendingMessageDeleteList {
    pub messages_ids: Vec<PendingMessageId>,
}

/// Message order number in a conversation.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct MessageNumber {
    pub message_number: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct UpdateMessageViewStatus {
    /// Sender of the messages.
    pub account_id_sender: AccountId,
    /// New message number for message view status.
    pub message_number: MessageNumber,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct SendMessageToAccount {
    /// Receiver of the message.
    pub receiver: AccountId,
    pub message: String,
}
