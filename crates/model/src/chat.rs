use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*, sql_types::BigInt};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper, UnixTime};
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, AccountIdDb, AccountIdInternal, ClientId, ClientLocalId};

mod db_only;
pub use db_only::*;

mod sync_version;
pub use sync_version::*;

mod push_notifications;
pub use push_notifications::*;

mod public_key;
pub use public_key::*;

mod received_likes;
pub use received_likes::*;

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
    pub new_received_likes_count: NewReceivedLikesCount,
    pub next_received_like_id: ReceivedLikeId,
    pub received_likes_iterator_reset_received_like_id_previous: Option<ReceivedLikeId>,
    pub received_likes_iterator_reset_received_like_id: Option<ReceivedLikeId>,
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
    pub state_change_unix_time: Option<UnixTime>,
    pub account_id_sender: Option<AccountIdDb>,
    pub account_id_receiver: Option<AccountIdDb>,
    /// Message counter is incrementing for each message sent.
    /// It does not reset even if interaction state goes from
    /// blocked to empty.
    pub message_counter: i64,
    pub sender_latest_viewed_message: Option<MessageNumber>,
    pub receiver_latest_viewed_message: Option<MessageNumber>,
    pub included_in_received_new_likes_count: bool,
    pub received_like_id: Option<ReceivedLikeId>,
    pub account_id_previous_like_deleter: Option<AccountIdDb>,
}

impl AccountInteractionInternal {
    pub fn try_into_like(
        self,
        id_like_sender: AccountIdInternal,
        id_like_receiver: AccountIdInternal,
        received_like_id: ReceivedLikeId,
    ) -> Result<Self, AccountInteractionStateError> {
        let target = AccountInteractionState::Like;
        let state = self.state_number;
        match state {
            AccountInteractionState::Empty => Ok(Self {
                state_number: target,
                state_change_unix_time: Some(UnixTime::current_time()),
                account_id_sender: Some(id_like_sender.into_db_id()),
                account_id_receiver: Some(id_like_receiver.into_db_id()),
                sender_latest_viewed_message: None,
                receiver_latest_viewed_message: None,
                included_in_received_new_likes_count: true,
                received_like_id: Some(received_like_id),
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
                state_change_unix_time: Some(UnixTime::current_time()),
                sender_latest_viewed_message: Some(MessageNumber::default()),
                receiver_latest_viewed_message: Some(MessageNumber::default()),
                included_in_received_new_likes_count: false,
                received_like_id: None,
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
                state_change_unix_time: Some(UnixTime::current_time()),
                account_id_sender: Some(id_block_sender.into_db_id()),
                account_id_receiver: Some(id_block_receiver.into_db_id()),
                sender_latest_viewed_message: None,
                receiver_latest_viewed_message: None,
                included_in_received_new_likes_count: false,
                received_like_id: None,
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
                state_change_unix_time: Some(UnixTime::current_time()),
                account_id_sender: None,
                account_id_receiver: None,
                sender_latest_viewed_message: None,
                receiver_latest_viewed_message: None,
                included_in_received_new_likes_count: false,
                received_like_id: None,
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
    pub sender_client_id: ClientId,
    pub sender_client_local_id: ClientLocalId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct SentLikesPage {
    /// This version can be sent to the server when WebSocket protocol
    /// data sync is happening.
    pub version: SentLikesSyncVersion,
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

#[derive(Debug, Clone)]
pub struct PendingMessageIdInternal {
    /// Sender of the message.
    pub sender: AccountIdInternal,
    pub mn: MessageNumber,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct PendingMessageAcknowledgementList {
    pub ids: Vec<PendingMessageId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct SentMessageId {
    pub c: ClientId,
    pub l: ClientLocalId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct SentMessageIdList {
    pub ids: Vec<SentMessageId>,
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
    #[serde(serialize_with = "client_id_as_i64", deserialize_with = "client_id_from_i64")]
    #[param(value_type = i64)]
    pub client_id: ClientId,
    #[serde(serialize_with = "client_local_id_as_i64", deserialize_with = "client_local_id_from_i64")]
    #[param(value_type = i64)]
    pub client_local_id: ClientLocalId,
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

pub fn client_id_as_i64<
    S: Serializer,
>(
    value: &ClientId,
    s: S,
) -> Result<S::Ok, S::Error> {
    value.id.serialize(s)
}

pub fn client_local_id_as_i64<
    S: Serializer,
>(
    value: &ClientLocalId,
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

pub fn client_id_from_i64<
    'de,
    D: Deserializer<'de>,
>(
    d: D,
) -> Result<ClientId, D::Error> {
    i64::deserialize(d).map(|id| ClientId { id })
}

pub fn client_local_id_from_i64<
    'de,
    D: Deserializer<'de>,
>(
    d: D,
) -> Result<ClientLocalId, D::Error> {
    i64::deserialize(d).map(|id| ClientLocalId { id })
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
    pub error_too_many_receiver_acknowledgements_missing: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_too_many_sender_acknowledgements_missing: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_receiver_public_key_outdated: bool,
}

impl SendMessageResult {
    pub fn is_err(&self) -> bool {
        self.error_too_many_receiver_acknowledgements_missing ||
        self.error_too_many_sender_acknowledgements_missing ||
        self.error_receiver_public_key_outdated
    }

    pub fn too_many_receiver_acknowledgements_missing() -> Self {
        Self {
            error_too_many_receiver_acknowledgements_missing: true,
            ..Self::default()
        }
    }

    pub fn too_many_sender_acknowledgements_missing() -> Self {
        Self {
            error_too_many_sender_acknowledgements_missing: true,
            ..Self::default()
        }
    }

    pub fn public_key_outdated() -> Self {
        Self {
            error_receiver_public_key_outdated: true,
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
pub struct DeleteLikeResult {
    /// The account tracking for delete like only tracks the latest deleter
    /// account, so it is possible that this error resets if delete like
    /// target account likes and removes the like.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_delete_already_done_before: bool,
}

impl DeleteLikeResult {
    pub fn success() -> Self {
        Self {
            error_delete_already_done_before: false,
        }
    }

    pub fn error_delete_already_done_once_before() -> Self {
        Self {
            error_delete_already_done_before: true
        }
    }
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

pub struct NewPendingMessageValues {
    pub unix_time: UnixTime,
    pub message_number: MessageNumber,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct SendLikeResult {
    pub status: Option<LimitedActionStatus>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_already_liked: bool,
}

impl SendLikeResult {
    pub fn successful(status: LimitedActionStatus) -> Self {
        Self {
            status: Some(status),
            error_already_liked: false,
        }
    }

    pub fn error_already_liked() -> Self {
        Self {
            status: None,
            error_already_liked: true,
        }
    }
}
