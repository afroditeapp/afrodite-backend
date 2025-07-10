use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::{UnixTime, diesel_i64_wrapper};
use utoipa::{IntoParams, ToSchema};

use super::sync_version_wrappers;
use crate::{AccountId, AccountIdDb, AccountIdInternal};

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

impl PublicKeyId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }
}

diesel_i64_wrapper!(PublicKeyId);

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
    ReceivedBlocksSyncVersion,
    /// Sync version for new received likes count
    ReceivedLikesSyncVersion,
    SentBlocksSyncVersion,
    SentLikesSyncVersion,
    MatchesSyncVersion,
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
    pub fn new(count: i64) -> Self {
        Self { c: count }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.c
    }

    /// Return new incremented value using `saturated_add`.
    pub fn increment(&self) -> Self {
        Self {
            c: self.c.saturating_add(1),
        }
    }

    /// Return new decremented value using `max(0, value - 1)`.
    pub fn decrement(&self) -> Self {
        Self {
            c: i64::max(0, self.c - 1),
        }
    }
}

diesel_i64_wrapper!(NewReceivedLikesCount);

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, ToSchema)]
pub struct ChatMessageReport {
    pub sender: AccountId,
    pub receiver: AccountId,
    pub message_time: UnixTime,
    pub message_number: MessageNumber,
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
}

#[derive(Debug, Clone)]
pub struct PendingMessageIdInternal {
    /// Sender of the message.
    pub sender: AccountIdInternal,
    /// Receiver of the message.
    pub receiver: AccountIdDb,
    pub mn: MessageNumber,
}

pub struct PendingMessageIdInternalAndMessageTime {
    pub id: PendingMessageIdInternal,
    pub time: UnixTime,
}
