use base64::Engine;
use diesel::prelude::*;
use model::{
    ConversationId, DailyLikesLeftSyncVersion, MatchId, MessageId, NewReceivedLikesCount,
    NewReceivedLikesCountResult, PublicKeyId, ReceivedLikeId, ReceivedLikesSyncVersion, UnixTime,
};
use model_server_data::LimitedActionStatus;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, AccountIdDb, ClientLocalId};

mod public_key;
pub use public_key::*;

mod received_likes;
pub use received_likes::*;

mod matches;
pub use matches::*;

mod report;
pub use report::*;

mod message;
pub use message::*;

mod video_call;
pub use video_call::*;

#[derive(Debug, Clone, Default, Serialize, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::chat_state)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct ChatStateRaw {
    pub received_likes_sync_version: ReceivedLikesSyncVersion,
    pub new_received_likes_count: NewReceivedLikesCount,
    pub next_received_like_id: ReceivedLikeId,
    pub next_conversation_id: ConversationId,
}

impl ChatStateRaw {
    pub fn new_received_likes_info(&self) -> NewReceivedLikesCountResult {
        NewReceivedLikesCountResult {
            v: self.received_likes_sync_version,
            c: self.new_received_likes_count,
            l: self.next_received_like_id.next_id_to_latest_used_id(),
            h: false,
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
    pub message_id: MessageId,
    pub message_bytes: Vec<u8>,
    pub sender_client_local_id: ClientLocalId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct SentBlocksPage {
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct PendingMessageId {
    /// Sender of the message.
    pub sender: AccountId,
    pub m: MessageId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct PendingMessageAcknowledgementList {
    pub ids: Vec<PendingMessageId>,
    /// Change sender's messages to delivered state
    #[serde(default, skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub change_to_delivered: bool,
}

fn value_is_true(v: &bool) -> bool {
    *v
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct MessageSeenList {
    pub ids: Vec<PendingMessageId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct SentMessageIdList {
    pub ids: Vec<ClientLocalId>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct UpdateMessageViewStatus {
    /// Sender of the messages.
    pub sender: AccountId,
    /// New message ID for message view status.
    pub m: MessageId,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, IntoParams)]
pub struct SendMessageToAccountParams {
    #[serde(
        serialize_with = "public_key_id_as_i64",
        deserialize_with = "public_key_id_from_i64"
    )]
    #[param(value_type = i64)]
    pub sender_public_key_id: PublicKeyId,
    /// Receiver of the message.
    #[serde(
        serialize_with = "account_id_as_string",
        deserialize_with = "account_id_from_uuid"
    )]
    #[param(value_type = String)]
    pub receiver: AccountId,
    /// Message receiver's public key ID for check
    /// to prevent sending message encrypted with outdated
    /// public key.
    #[serde(
        serialize_with = "public_key_id_as_i64",
        deserialize_with = "public_key_id_from_i64"
    )]
    #[param(value_type = i64)]
    pub receiver_public_key_id: PublicKeyId,
    #[serde(
        serialize_with = "client_local_id_as_string",
        deserialize_with = "client_local_id_from_string"
    )]
    #[param(value_type = String)]
    pub client_local_id: ClientLocalId,
}

pub fn account_id_as_string<S: Serializer>(value: &AccountId, s: S) -> Result<S::Ok, S::Error> {
    value.aid.serialize(s)
}

pub fn public_key_id_as_i64<S: Serializer>(value: &PublicKeyId, s: S) -> Result<S::Ok, S::Error> {
    value.id.serialize(s)
}

pub fn client_local_id_as_string<S: Serializer>(
    value: &ClientLocalId,
    s: S,
) -> Result<S::Ok, S::Error> {
    value.id().serialize(s)
}

pub fn account_id_from_uuid<'de, D: Deserializer<'de>>(d: D) -> Result<AccountId, D::Error> {
    simple_backend_utils::UuidBase64Url::deserialize(d)
        .map(|account_id| AccountId { aid: account_id })
}

pub fn public_key_id_from_i64<'de, D: Deserializer<'de>>(d: D) -> Result<PublicKeyId, D::Error> {
    i64::deserialize(d).map(|id| PublicKeyId { id })
}

pub fn client_local_id_from_string<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<ClientLocalId, D::Error> {
    simple_backend_utils::UuidBase64Url::deserialize(d).map(ClientLocalId::new)
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct SendMessageResult {
    /// Base64 encoded PGP signed message containing [SignedMessageData].
    d: Option<String>,
    // Errors
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_too_many_receiver_acknowledgements_missing: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_too_many_sender_acknowledgements_missing: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_sender_public_key_outdated: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_receiver_public_key_outdated: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_receiver_blocked_sender_or_receiver_not_found: bool,
}

impl SendMessageResult {
    pub fn is_err(&self) -> bool {
        self.error
    }

    pub fn too_many_receiver_acknowledgements_missing() -> Self {
        Self {
            error: true,
            error_too_many_receiver_acknowledgements_missing: true,
            ..Self::default()
        }
    }

    pub fn too_many_sender_acknowledgements_missing() -> Self {
        Self {
            error: true,
            error_too_many_sender_acknowledgements_missing: true,
            ..Self::default()
        }
    }

    pub fn sender_public_key_outdated() -> Self {
        Self {
            error: true,
            error_sender_public_key_outdated: true,
            ..Self::default()
        }
    }

    pub fn receiver_public_key_outdated() -> Self {
        Self {
            error: true,
            error_receiver_public_key_outdated: true,
            ..Self::default()
        }
    }

    pub fn receiver_blocked_sender_or_receiver_not_found() -> Self {
        Self {
            error: true,
            error_receiver_blocked_sender_or_receiver_not_found: true,
            ..Self::default()
        }
    }

    pub fn successful(data: Vec<u8>) -> Self {
        Self {
            d: Some(base64::engine::general_purpose::STANDARD.encode(data)),
            ..Self::default()
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct GetSentMessage {
    /// Base64 encoded PGP signed message containing [SignedMessageData].
    #[allow(dead_code)]
    data: String,
}

impl GetSentMessage {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: base64::engine::general_purpose::STANDARD.encode(data),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct SendLikeResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<LimitedActionStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_likes_left: Option<DailyLikesLeft>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_account_interaction_state_mismatch: Option<CurrentAccountInteractionState>,
}

impl SendLikeResult {
    pub fn successful(status: LimitedActionStatus, daily_likes_left: DailyLikesLeft) -> Self {
        Self {
            status: Some(status),
            daily_likes_left: Some(daily_likes_left),
            error_account_interaction_state_mismatch: None,
        }
    }

    pub fn error_account_interaction_state_mismatch(state: CurrentAccountInteractionState) -> Self {
        Self {
            status: None,
            daily_likes_left: None,
            error_account_interaction_state_mismatch: Some(state),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub enum CurrentAccountInteractionState {
    Empty,
    LikeSent,
    LikeReceived,
    Match,
    BlockSent,
}

pub const CHAT_GLOBAL_STATE_ROW_TYPE: i32 = 0;

/// Global state for account component
#[derive(Debug, Default, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = crate::schema::chat_global_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct ChatGlobalState {
    pub next_match_id: MatchId,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::daily_likes_left)]
#[diesel(check_for_backend(crate::Db))]
pub struct DailyLikesLeftInternal {
    pub sync_version: DailyLikesLeftSyncVersion,
    pub likes_left: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_limit_reset_unix_time: Option<UnixTime>,
}

#[derive(Serialize, ToSchema)]
pub struct DailyLikesLeft {
    /// This value can be ignored when like sending limit is not enabled.
    pub likes: i16,
    pub version: DailyLikesLeftSyncVersion,
}

impl From<DailyLikesLeftInternal> for DailyLikesLeft {
    fn from(value: DailyLikesLeftInternal) -> Self {
        Self {
            likes: value.likes_left,
            version: value.sync_version,
        }
    }
}
