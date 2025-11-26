use base64::{Engine, prelude::BASE64_STANDARD};
use diesel::{
    Selectable,
    deserialize::FromSqlRow,
    expression::AsExpression,
    prelude::Queryable,
    sql_types::{BigInt, Binary},
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{UnixTime, diesel_i64_wrapper};
use simple_backend_utils::diesel_uuid_wrapper;
use utoipa::{IntoParams, ToSchema};

use super::sync_version_wrappers;
use crate::{AccountId, AccountIdDb, AccountIdInternal, LastSeenTime};

mod interaction;
pub use interaction::*;

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
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct PublicKeyId {
    pub id: i64,
}

impl TryFrom<i64> for PublicKeyId {
    type Error = String;

    fn try_from(id: i64) -> Result<Self, Self::Error> {
        Ok(Self { id })
    }
}

impl AsRef<i64> for PublicKeyId {
    fn as_ref(&self) -> &i64 {
        &self.id
    }
}

diesel_i64_wrapper!(PublicKeyId);

/// Message UUID which sender generates
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
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = Binary)]
pub struct MessageUuid {
    id: simple_backend_utils::UuidBase64Url,
}

impl MessageUuid {
    pub fn new(id: simple_backend_utils::UuidBase64Url) -> Self {
        Self { id }
    }

    pub fn id(&self) -> simple_backend_utils::UuidBase64Url {
        self.id
    }
}

impl TryFrom<simple_backend_utils::UuidBase64Url> for MessageUuid {
    type Error = String;

    fn try_from(id: simple_backend_utils::UuidBase64Url) -> Result<Self, Self::Error> {
        Ok(Self { id })
    }
}

impl AsRef<simple_backend_utils::UuidBase64Url> for MessageUuid {
    fn as_ref(&self) -> &simple_backend_utils::UuidBase64Url {
        &self.id
    }
}

diesel_uuid_wrapper!(MessageUuid);

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct NewMessageNotificationList {
    pub v: Vec<NewMessageNotification>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct NewMessageNotification {
    pub a: AccountId,
    pub c: ConversationId,
    /// Message count
    pub m: i64,
}

sync_version_wrappers!(
    /// Sync version for new received likes count
    ReceivedLikesSyncVersion,
    DailyLikesLeftSyncVersion,
);

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
pub struct NewReceivedLikesCount {
    pub c: i64,
}

impl NewReceivedLikesCount {
    /// Return new incremented value using `saturated_add`.
    pub fn increment(&self) -> Self {
        Self {
            c: self.c.saturating_add(1),
        }
    }
}

impl TryFrom<i64> for NewReceivedLikesCount {
    type Error = String;

    fn try_from(count: i64) -> Result<Self, Self::Error> {
        Ok(Self { c: count })
    }
}

impl AsRef<i64> for NewReceivedLikesCount {
    fn as_ref(&self) -> &i64 {
        &self.c
    }
}

diesel_i64_wrapper!(NewReceivedLikesCount);

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, ToSchema)]
pub struct ChatMessageReport {
    pub sender: AccountId,
    pub receiver: AccountId,
    pub message_time: UnixTime,
    pub message_id: MessageId,
    /// Message without encryption and signing
    pub message_base64: String,
}

#[derive(Deserialize, ToSchema)]
pub struct GetChatMessageReports {
    pub creator: AccountId,
    pub target: AccountId,
    pub only_not_processed: bool,
}

pub struct GetChatMessageReportsInternal {
    pub creator: AccountIdInternal,
    pub target: AccountIdInternal,
    pub only_not_processed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct NewReceivedLikesCountResult {
    pub v: ReceivedLikesSyncVersion,
    pub c: NewReceivedLikesCount,
    /// Latest received like in use. Client can use this
    /// to check should received likes be refreshed.
    pub l: ReceivedLikeId,
    /// If true, client should not show the notification
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub h: bool,
}

#[derive(Debug, Clone)]
pub struct PendingMessageIdInternal {
    /// Sender of the message.
    pub sender: AccountIdInternal,
    /// Receiver of the message.
    pub receiver: AccountIdDb,
    pub m: MessageId,
}
pub struct PendingMessageIdInternalAndMessageTime {
    pub id: PendingMessageIdInternal,
    pub time: UnixTime,
}

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::pending_messages)]
#[diesel(check_for_backend(crate::Db))]
pub struct PendingMessageRaw {
    pub id: i64,
    pub account_id_sender: AccountIdDb,
    pub account_id_receiver: AccountIdDb,
    pub sender_acknowledgement: bool,
    pub receiver_acknowledgement: bool,
    pub receiver_push_notification_sent: bool,
    pub receiver_email_notification_sent: bool,
    pub message_unix_time: UnixTime,
    pub message_id: MessageId,
    pub message_uuid: MessageUuid,
}

#[derive(Serialize)]
pub struct DataExportPendingMessage {
    message_unix_time: UnixTime,
    message_id: MessageId,
    message_uuid: MessageUuid,
    message_bytes_base64: String,
}

impl DataExportPendingMessage {
    pub fn new(message: PendingMessageRaw, message_bytes: Vec<u8>) -> Self {
        Self {
            message_unix_time: message.message_unix_time,
            message_id: message.message_id,
            message_uuid: message.message_uuid,
            message_bytes_base64: BASE64_STANDARD.encode(message_bytes),
        }
    }
}

#[derive(Serialize)]
pub struct AdminDataExportPendingMessage {
    pending_message: PendingMessageRaw,
    message_bytes_base64: String,
}

impl AdminDataExportPendingMessage {
    pub fn new(pending_message: PendingMessageRaw, message_bytes: Vec<u8>) -> Self {
        Self {
            pending_message,
            message_bytes_base64: BASE64_STANDARD.encode(message_bytes),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CheckOnlineStatusResponse {
    pub a: AccountId,
    pub l: LastSeenTime,
}
