use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*, sql_types::BigInt};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper, UnixTime};
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, AccountIdDb, AccountIdInternal};

mod db_only;
pub use db_only::*;

mod sync_version;
pub use sync_version::*;

mod push_notifications;
pub use push_notifications::*;

mod public_key;
pub use public_key::*;

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
    pub state_number: AccountInteractionState,
    pub account_id_sender: Option<AccountIdDb>,
    pub account_id_receiver: Option<AccountIdDb>,
    /// Message counter is incrementing for each message sent.
    /// It does not reset even if interaction state goes from
    /// blocked to empty.
    pub message_counter: i64,
    pub sender_latest_viewed_message: Option<MessageNumber>,
    pub receiver_latest_viewed_message: Option<MessageNumber>,
    pub sender_next_message_id: SenderMessageId,
    pub receiver_next_message_id: SenderMessageId,
}

impl AccountInteractionInternal {
    pub fn try_into_like(
        self,
        id_like_sender: AccountIdInternal,
        id_like_receiver: AccountIdInternal,
    ) -> Result<Self, AccountInteractionStateError> {
        let target = AccountInteractionState::Like;
        let state = self.state_number;
        match state {
            AccountInteractionState::Empty => Ok(Self {
                state_number: target,
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
        let state = self.state_number;
        match state {
            AccountInteractionState::Like => Ok(Self {
                state_number: target,
                sender_latest_viewed_message: Some(MessageNumber::default()),
                receiver_latest_viewed_message: Some(MessageNumber::default()),
                sender_next_message_id: SenderMessageId::default(),
                receiver_next_message_id: SenderMessageId::default(),
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
        let state = self.state_number;
        match state {
            AccountInteractionState::Empty
            | AccountInteractionState::Like
            | AccountInteractionState::Match => Ok(Self {
                state_number: AccountInteractionState::Block,
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
        let state = self.state_number;
        match state {
            AccountInteractionState::Block | AccountInteractionState::Like => Ok(Self {
                state_number: target,
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
        self.state_number == AccountInteractionState::Empty
    }

    pub fn is_like(&self) -> bool {
        self.state_number == AccountInteractionState::Like
    }

    pub fn is_match(&self) -> bool {
        self.state_number == AccountInteractionState::Match
    }

    pub fn is_blocked(&self) -> bool {
        self.state_number == AccountInteractionState::Block
    }

    /// Get next expected message ID for the account
    pub fn next_expected_message_id_mut(
        &mut self,
        account: AccountIdDb,
    ) -> Option<&mut SenderMessageId> {
        if self.account_id_receiver == Some(account) {
            Some(&mut self.receiver_next_message_id)
        } else if self.account_id_sender == Some(account) {
            Some(&mut self.sender_next_message_id)
        } else {
            None
        }
    }

    /// Get next expected message ID for the account
    pub fn next_expected_message_id(
        &self,
        account: AccountIdDb,
    ) -> Option<&SenderMessageId> {
        if self.account_id_receiver == Some(account) {
            Some(&self.receiver_next_message_id)
        } else if self.account_id_sender == Some(account) {
            Some(&self.sender_next_message_id)
        } else {
            None
        }
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
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = BigInt)]
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

diesel_i64_try_from!(AccountInteractionState);

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::pending_messages)]
#[diesel(check_for_backend(crate::Db))]
pub struct PendingMessageInternal {
    pub id: i64,
    pub account_id_sender: AccountIdDb,
    pub account_id_receiver: AccountIdDb,
    pub unix_time: UnixTime,
    pub message_number: MessageNumber,
    pub message_bytes: Vec<u8>,
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

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct PendingMessage {
    pub id: PendingMessageId,
    /// Unix time when server received the message.
    pub unix_time: UnixTime,
}

#[derive(Debug, Clone)]
pub struct PendingMessageAndMessageData {
    pub pending_message: PendingMessage,
    pub message: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct PendingMessageId {
    /// Sender of the message.
    pub sender: AccountId,
    pub mn: MessageNumber,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct PendingMessageDeleteList {
    pub ids: Vec<PendingMessageId>,
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
    pub mn: i64,
}

impl MessageNumber {
    pub fn new(id: i64) -> Self {
        Self { mn: id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.mn
    }
}

diesel_i64_wrapper!(MessageNumber);

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct UpdateMessageViewStatus {
    /// Sender of the messages.
    pub sender: AccountId,
    /// New message number for message view status.
    pub mn: MessageNumber,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, IntoParams)]
pub struct SendMessageToAccountParams {
    /// Receiver of the message.
    #[serde(serialize_with = "account_id_as_uuid", deserialize_with = "account_id_from_uuid")]
    #[param(value_type = uuid::Uuid)]
    pub receiver: AccountId,
    /// Message receiver's public key ID for check
    /// to prevent sending message encrypted with outdated
    /// public key.
    #[serde(serialize_with = "public_key_id_as_i64", deserialize_with = "public_key_id_from_i64")]
    #[param(value_type = i64)]
    pub receiver_public_key_id: PublicKeyId,
    #[serde(serialize_with = "public_key_version_as_i64", deserialize_with = "public_key_version_from_i64")]
    #[param(value_type = i64)]
    pub receiver_public_key_version: PublicKeyVersion,
    #[serde(serialize_with = "sender_message_id_as_i64", deserialize_with = "sender_message_id_from_i64")]
    #[param(value_type = i64)]
    pub sender_message_id: SenderMessageId,
}

pub fn account_id_as_uuid<
    S: Serializer,
>(
    value: &AccountId,
    s: S,
) -> Result<S::Ok, S::Error> {
    value.aid.serialize(s)
}

pub fn public_key_id_as_i64<
    S: Serializer,
>(
    value: &PublicKeyId,
    s: S,
) -> Result<S::Ok, S::Error> {
    value.id.serialize(s)
}

pub fn public_key_version_as_i64<
    S: Serializer,
>(
    value: &PublicKeyVersion,
    s: S,
) -> Result<S::Ok, S::Error> {
    value.version.serialize(s)
}

pub fn sender_message_id_as_i64<
    S: Serializer,
>(
    value: &SenderMessageId,
    s: S,
) -> Result<S::Ok, S::Error> {
    value.id.serialize(s)
}

pub fn account_id_from_uuid<
    'de,
    D: Deserializer<'de>,
>(
    d: D,
) -> Result<AccountId, D::Error> {
    uuid::Uuid::deserialize(d).map(|account_id| AccountId { aid: account_id })
}

pub fn public_key_id_from_i64<
    'de,
    D: Deserializer<'de>,
>(
    d: D,
) -> Result<PublicKeyId, D::Error> {
    i64::deserialize(d).map(|id| PublicKeyId { id })
}

pub fn public_key_version_from_i64<
    'de,
    D: Deserializer<'de>,
>(
    d: D,
) -> Result<PublicKeyVersion, D::Error> {
    i64::deserialize(d).map(|version| PublicKeyVersion { version })
}

pub fn sender_message_id_from_i64<
    'de,
    D: Deserializer<'de>,
>(
    d: D,
) -> Result<SenderMessageId, D::Error> {
    i64::deserialize(d).map(|id| SenderMessageId { id })
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct SendMessageResult {
    /// None if error happened
    ut: Option<UnixTime>,
    /// None if error happened
    mn: Option<MessageNumber>,
    // Errors
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_too_many_pending_messages: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_receiver_public_key_outdated: bool,
    pub error_sender_message_id_was_not_expected_id: Option<SenderMessageId>,
}

impl SendMessageResult {
    pub fn is_err(&self) -> bool {
        self.error_too_many_pending_messages ||
        self.error_receiver_public_key_outdated ||
        self.error_sender_message_id_was_not_expected_id.is_some()
    }

    pub fn too_many_pending_messages() -> Self {
        Self {
            error_too_many_pending_messages: true,
            ..Self::default()
        }
    }

    pub fn public_key_outdated() -> Self {
        Self {
            error_receiver_public_key_outdated: true,
            ..Self::default()
        }
    }

    pub fn sender_message_id_was_not_expected_id(
        expected_id: SenderMessageId,
    ) -> Self {
        Self {
            error_sender_message_id_was_not_expected_id: Some(expected_id),
            ..Self::default()
        }
    }

    pub fn successful(values: NewPendingMessageValues) -> Self {
        Self {
            ut: Some(values.unix_time),
            mn: Some(values.message_number),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct LimitedActionResult {
    pub status: LimitedActionStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub enum LimitedActionStatus {
    /// Action completed successfully.
    Success,
    /// Action completed successfully but the action limit was reached.
    SuccessAndLimitReached,
    /// Action failed because the action limit is already reached.
    FailureLimitAlreadyReached,
}

/// Conversation message counter located on the server which only message sender
/// owns (conversation can have 2 senders, so there is two counters).
///
/// The server increments the ID automatically when message is sent. The server
/// resets the ID when account interaction is changed to match state. Also the
/// client can reset the counter as it might go out of sync for example
/// when the account is changed to different device.
#[derive(
    Debug,
    Serialize,
    Deserialize,
    ToSchema,
    Clone,
    Eq,
    Hash,
    PartialEq,
    IntoParams,
    Copy,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct SenderMessageId {
    pub id: i64,
}

impl SenderMessageId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }

    pub fn increment(&self) -> Self {
        Self {
            id: self.id.wrapping_add(1),
        }
    }
}

diesel_i64_wrapper!(SenderMessageId);

pub struct NewPendingMessageValues {
    pub unix_time: UnixTime,
    pub message_number: MessageNumber,
}
