use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_wrapper, UnixTime};
use utoipa::ToSchema;

use crate::{AccountId, AccountIdDb, AccountIdInternal, SyncVersionUtils};

mod sync_version;
pub use sync_version::*;

mod push_notifications;
pub use push_notifications::*;

#[derive(Debug, Clone, Default, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::chat_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct ChatStateRaw {
    pub received_blocks_sync_version: ReceivedBlocksSyncVersion,
    pub received_likes_sync_version: ReceivedLikesSyncVersion,
    pub sent_blocks_sync_version: SentBlocksSyncVersion,
    pub sent_likes_sync_version: SentLikesSyncVersion,
    pub matches_sync_version: MatchesSyncVersion,
    pub pending_notification: PendingNotification,
    pub fcm_notification_sent: bool,
    pub fcm_device_token: Option<FcmDeviceToken>,
}

#[derive(Debug, Clone, Copy)]
pub enum AccountInteractionStateError {
    WrongStateNumber(i64),
    Transition {
        from: AccountInteractionState,
        to: AccountInteractionState,
    },
}
impl AccountInteractionStateError {
    pub fn wrong_state_number(number: i64) -> Self {
        Self::WrongStateNumber(number)
    }
    pub fn transition(from: AccountInteractionState, to: AccountInteractionState) -> Self {
        Self::Transition { from, to }
    }
}
impl std::fmt::Display for AccountInteractionStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountInteractionStateError::WrongStateNumber(number) => {
                write!(f, "Wrong state number: {}", number)
            }
            AccountInteractionStateError::Transition { from, to } => {
                write!(
                    f,
                    "State transition from {:?} to {:?} is not allowed",
                    from, to
                )
            }
        }
    }
}
impl std::error::Error for AccountInteractionStateError {}

#[derive(Debug, Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::account_interaction)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountInteractionInternal {
    pub id: i64,
    pub state_number: i64,
    pub account_id_sender: Option<AccountIdDb>,
    pub account_id_receiver: Option<AccountIdDb>,
    /// Message counter is incrementing for each message sent.
    /// It does not reset even if interaction state goes from
    /// blocked to empty.
    pub message_counter: i64,
    pub sender_latest_viewed_message: Option<MessageNumber>,
    pub receiver_latest_viewed_message: Option<MessageNumber>,
}

impl AccountInteractionInternal {
    pub fn try_into_like(
        self,
        id_like_sender: AccountIdInternal,
        id_like_receiver: AccountIdInternal,
    ) -> Result<Self, AccountInteractionStateError> {
        let target = AccountInteractionState::Like;
        let state = AccountInteractionState::try_from(self.state_number)?;
        match state {
            AccountInteractionState::Empty => Ok(Self {
                state_number: target as i64,
                account_id_sender: Some(id_like_sender.into_db_id()),
                account_id_receiver: Some(id_like_receiver.into_db_id()),
                sender_latest_viewed_message: None,
                receiver_latest_viewed_message: None,
                ..self
            }),
            AccountInteractionState::Like => Ok(self),
            AccountInteractionState::Match | AccountInteractionState::Block => {
                Err(AccountInteractionStateError::transition(state, target))
            }
        }
    }

    pub fn try_into_match(self) -> Result<Self, AccountInteractionStateError> {
        let target = AccountInteractionState::Match;
        let state = AccountInteractionState::try_from(self.state_number)?;
        match state {
            AccountInteractionState::Like => Ok(Self {
                state_number: target as i64,
                sender_latest_viewed_message: Some(MessageNumber::default()),
                receiver_latest_viewed_message: Some(MessageNumber::default()),
                ..self
            }),
            AccountInteractionState::Match => Ok(self),
            AccountInteractionState::Empty | AccountInteractionState::Block => {
                Err(AccountInteractionStateError::transition(state, target))
            }
        }
    }

    pub fn try_into_block(
        self,
        id_block_sender: AccountIdInternal,
        id_block_receiver: AccountIdInternal,
    ) -> Result<Self, AccountInteractionStateError> {
        let state = AccountInteractionState::try_from(self.state_number)?;
        match state {
            AccountInteractionState::Empty
            | AccountInteractionState::Like
            | AccountInteractionState::Match => Ok(Self {
                state_number: AccountInteractionState::Block as i64,
                account_id_sender: Some(id_block_sender.into_db_id()),
                account_id_receiver: Some(id_block_receiver.into_db_id()),
                sender_latest_viewed_message: None,
                receiver_latest_viewed_message: None,
                ..self
            }),
            AccountInteractionState::Block => Ok(self),
        }
    }

    pub fn try_into_empty(self) -> Result<Self, AccountInteractionStateError> {
        let target = AccountInteractionState::Empty;
        let state = AccountInteractionState::try_from(self.state_number)?;
        match state {
            AccountInteractionState::Block | AccountInteractionState::Like => Ok(Self {
                state_number: target as i64,
                account_id_sender: None,
                account_id_receiver: None,
                sender_latest_viewed_message: None,
                receiver_latest_viewed_message: None,
                ..self
            }),
            AccountInteractionState::Empty => Ok(self),
            AccountInteractionState::Match => {
                Err(AccountInteractionStateError::transition(state, target))
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.state_number == AccountInteractionState::Empty as i64
    }

    pub fn is_like(&self) -> bool {
        self.state_number == AccountInteractionState::Like as i64
    }

    pub fn is_match(&self) -> bool {
        self.state_number == AccountInteractionState::Match as i64
    }

    pub fn is_blocked(&self) -> bool {
        self.state_number == AccountInteractionState::Block as i64
    }
}

/// Account interaction states
///
/// Possible state transitions:
/// - Empty -> Like -> Match -> Block
/// - Empty -> Like -> Block
/// - Empty -> Block
/// - Block -> Empty
/// - Like -> Empty
#[derive(Debug, Clone, Copy)]
pub enum AccountInteractionState {
    Empty = 0,
    Like = 1,
    Match = 2,
    Block = 3,
}

impl TryFrom<i64> for AccountInteractionState {
    type Error = AccountInteractionStateError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Empty),
            1 => Ok(Self::Like),
            2 => Ok(Self::Match),
            3 => Ok(Self::Block),
            _ => Err(AccountInteractionStateError::WrongStateNumber(value)),
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
    pub unix_time: UnixTime,
    pub message_number: MessageNumber,
    pub message_text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct SentLikesPage {
    /// This version can be sent to the server when WebSocket protocol
    /// data sync is happening.
    pub version: SentLikesSyncVersion,
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct ReceivedLikesPage {
    /// This version can be sent to the server when WebSocket protocol
    /// data sync is happening.
    pub version: ReceivedLikesSyncVersion,
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct MatchesPage {
    /// This version can be sent to the server when WebSocket protocol
    /// data sync is happening.
    pub version: MatchesSyncVersion,
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct SentBlocksPage {
    /// This version can be sent to the server when WebSocket protocol
    /// data sync is happening.
    pub version: SentBlocksSyncVersion,
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct ReceivedBlocksPage {
    /// This version can be sent to the server when WebSocket protocol
    /// data sync is happening.
    pub version: ReceivedBlocksSyncVersion,
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct PendingMessage {
    pub id: PendingMessageId,
    /// Unix time when server received the message.
    pub unix_time: UnixTime,
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

// TODO(prod): change messages_ids to message_ids
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct PendingMessageDeleteList {
    pub messages_ids: Vec<PendingMessageId>,
}

/// Message order number in a conversation.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct MessageNumber {
    pub message_number: i64,
}

impl MessageNumber {
    pub fn new(id: i64) -> Self {
        Self { message_number: id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.message_number
    }
}

diesel_i64_wrapper!(MessageNumber);

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
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
