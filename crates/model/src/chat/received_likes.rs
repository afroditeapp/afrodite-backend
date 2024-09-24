use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::ToSchema;

use crate::AccountId;

use super::ReceivedLikesSyncVersion;


/// Session ID type for received likes iterator so that client can detect
/// server restarts and ask user to refresh received likes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReceivedLikesIteratorSessionIdInternal {
    id: uuid::Uuid,
}

impl ReceivedLikesIteratorSessionIdInternal {
    /// Current implementation uses UUID. Only requirement for this
    /// type is that next one should be different than the previous.
    pub fn create_random() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
        }
    }
}

/// Session ID type for received likes iterator so that client can detect
/// server restarts and ask user to refresh received likes.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ReceivedLikesIteratorSessionId {
    id: String,
}

impl From<ReceivedLikesIteratorSessionIdInternal> for ReceivedLikesIteratorSessionId {
    fn from(value: ReceivedLikesIteratorSessionIdInternal) -> Self {
        Self {
            id: value.id.hyphenated().to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct NewReceivedLikesAvailableResult {
    pub v: ReceivedLikesSyncVersion,
    pub c: NewReceivedLikesCount,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResetReceivedLikesIteratorResult {
    pub v: ReceivedLikesSyncVersion,
    pub c: NewReceivedLikesCount,
    pub s: ReceivedLikesIteratorSessionId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct ReceivedLikesPage {
    pub p: Vec<AccountId>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_invalid_iterator_session_id: bool,
}

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
        Self { c: self.c.saturating_add(1) }
    }
}

diesel_i64_wrapper!(NewReceivedLikesCount);
