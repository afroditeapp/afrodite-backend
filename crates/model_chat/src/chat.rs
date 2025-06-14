use base64::Engine;
use diesel::prelude::*;
use model::{
    MatchId, MatchesSyncVersion, MessageNumber, NewReceivedLikesCount, ProfileContentVersion, PublicKeyId, ReceivedBlocksSyncVersion, ReceivedLikeId, ReceivedLikesSyncVersion, SentBlocksSyncVersion, SentLikesSyncVersion
};
use model_server_data::{LastSeenTime, LimitedActionStatus, ProfileVersion};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, AccountIdDb, AccountIdInternal, ClientId, ClientLocalId};

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

#[derive(Debug, Clone, Default, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::chat_state)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct ChatStateRaw {
    pub received_blocks_sync_version: ReceivedBlocksSyncVersion,
    pub received_likes_sync_version: ReceivedLikesSyncVersion,
    pub sent_blocks_sync_version: SentBlocksSyncVersion,
    pub sent_likes_sync_version: SentLikesSyncVersion,
    pub matches_sync_version: MatchesSyncVersion,
    pub new_received_likes_count: NewReceivedLikesCount,
    pub next_received_like_id: ReceivedLikeId,
    pub received_like_id_at_received_likes_iterator_reset: Option<ReceivedLikeId>,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::pending_messages)]
#[diesel(check_for_backend(crate::Db))]
pub struct PendingMessageInternal {
    pub id: i64,
    pub account_id_sender: AccountIdDb,
    pub account_id_receiver: AccountIdDb,
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct UpdateMessageViewStatus {
    /// Sender of the messages.
    pub sender: AccountId,
    /// New message number for message view status.
    pub mn: MessageNumber,
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
        serialize_with = "client_id_as_i64",
        deserialize_with = "client_id_from_i64"
    )]
    #[param(value_type = i64)]
    pub client_id: ClientId,
    #[serde(
        serialize_with = "client_local_id_as_i64",
        deserialize_with = "client_local_id_from_i64"
    )]
    #[param(value_type = i64)]
    pub client_local_id: ClientLocalId,
}

pub fn account_id_as_string<S: Serializer>(value: &AccountId, s: S) -> Result<S::Ok, S::Error> {
    value.aid.serialize(s)
}

pub fn public_key_id_as_i64<S: Serializer>(value: &PublicKeyId, s: S) -> Result<S::Ok, S::Error> {
    value.id.serialize(s)
}

pub fn client_id_as_i64<S: Serializer>(value: &ClientId, s: S) -> Result<S::Ok, S::Error> {
    value.id.serialize(s)
}

pub fn client_local_id_as_i64<S: Serializer>(
    value: &ClientLocalId,
    s: S,
) -> Result<S::Ok, S::Error> {
    value.id.serialize(s)
}

pub fn account_id_from_uuid<'de, D: Deserializer<'de>>(d: D) -> Result<AccountId, D::Error> {
    simple_backend_utils::UuidBase64Url::deserialize(d)
        .map(|account_id| AccountId { aid: account_id })
}

pub fn public_key_id_from_i64<'de, D: Deserializer<'de>>(d: D) -> Result<PublicKeyId, D::Error> {
    i64::deserialize(d).map(|id| PublicKeyId { id })
}

pub fn client_id_from_i64<'de, D: Deserializer<'de>>(d: D) -> Result<ClientId, D::Error> {
    i64::deserialize(d).map(|id| ClientId { id })
}

pub fn client_local_id_from_i64<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<ClientLocalId, D::Error> {
    i64::deserialize(d).map(|id| ClientLocalId { id })
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct SendMessageResult {
    /// Base64 encoded PGP signed message containing [SignedMessageData].
    d: Option<String>,
    // Errors
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_too_many_receiver_acknowledgements_missing: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_too_many_sender_acknowledgements_missing: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_sender_public_key_outdated: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_receiver_public_key_outdated: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_receiver_blocked_sender_or_receiver_not_found: bool,
}

impl SendMessageResult {
    pub fn is_err(&self) -> bool {
        self.error_too_many_receiver_acknowledgements_missing
            || self.error_too_many_sender_acknowledgements_missing
            || self.error_sender_public_key_outdated
            || self.error_receiver_public_key_outdated
            || self.error_receiver_blocked_sender_or_receiver_not_found
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

    pub fn sender_public_key_outdated() -> Self {
        Self {
            error_sender_public_key_outdated: true,
            ..Self::default()
        }
    }

    pub fn receiver_public_key_outdated() -> Self {
        Self {
            error_receiver_public_key_outdated: true,
            ..Self::default()
        }
    }

    pub fn receiver_blocked_sender_or_receiver_not_found() -> Self {
        Self {
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
            data: base64::engine::general_purpose::STANDARD.encode(data)
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
    pub error_account_interaction_state_mismatch: Option<CurrentAccountInteractionState>,
}

impl DeleteLikeResult {
    pub fn success() -> Self {
        Self {
            error_delete_already_done_before: false,
            error_account_interaction_state_mismatch: None,
        }
    }

    pub fn error_delete_already_done_once_before() -> Self {
        Self {
            error_delete_already_done_before: true,
            error_account_interaction_state_mismatch: None,
        }
    }

    pub fn error_account_interaction_state_mismatch(state: CurrentAccountInteractionState) -> Self {
        Self {
            error_delete_already_done_before: false,
            error_account_interaction_state_mismatch: Some(state),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct SendLikeResult {
    pub status: Option<LimitedActionStatus>,
    /// None if the limit is disabled.
    pub daily_likes_left: Option<u8>,
    pub error_account_interaction_state_mismatch: Option<CurrentAccountInteractionState>,
}

impl SendLikeResult {
    pub fn successful(status: LimitedActionStatus, daily_likes_left: Option<u8>) -> Self {
        Self {
            status: Some(status),
            daily_likes_left,
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

pub const CHAT_GLOBAL_STATE_ROW_TYPE: i64 = 0;

/// Global state for account component
#[derive(Debug, Default, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = crate::schema::chat_global_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct ChatGlobalState {
    pub next_match_id: MatchId,
}

/// Similar as [model_server_data::ProfileLink] but for chat component.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ChatProfileLink {
    a: AccountId,
    /// This is optional because profile component owns it.
    p: Option<ProfileVersion>,
    /// This is optional because media component owns it.
    c: Option<ProfileContentVersion>,
    /// If the last seen time is not None, then it is Unix timestamp or -1 if
    /// the profile is currently online.
    l: Option<LastSeenTime>,
}

impl ChatProfileLink {
    pub fn new(
        id: AccountId,
        version: Option<ProfileVersion>,
        content_version: Option<ProfileContentVersion>,
        last_seen_time: Option<LastSeenTime>,
    ) -> Self {
        Self {
            a: id,
            p: version,
            c: content_version,
            l: last_seen_time,
        }
    }
}
