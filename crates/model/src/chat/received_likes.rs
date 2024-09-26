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
pub struct NewReceivedLikesCountResult {
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
    pub n: PageItemCountForNewLikes,
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

    /// Return new decremented value using `max(0, value - 1)`.
    pub fn decrement(&self) -> Self {
        Self { c: i64::max(0, self.c - 1) }
    }
}

diesel_i64_wrapper!(NewReceivedLikesCount);

/// Define how many returned profiles counted from the first page item are
/// new likes (interaction state changed to like after previous received likes
/// iterator reset).
///
/// NOTE: The current alogirthm for new likes count does not
/// handle the following case:
/// 1. time: 0, Iterator reset happens.
/// 2. time: 0, First page is returned.
/// 3. time: 0, New like is added.
/// 4. time: 1, Iterator reset happens.
/// 5. time: 1, First page is returned. The new like is not
///    added in the new likes count because
///    state_change_unix_time.gt(reset_time_previous)
///    is false.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
)]
pub struct PageItemCountForNewLikes {
    pub c: i64,
}

impl PageItemCountForNewLikes {
    /// Add another count using `saturating_add`
    pub fn merge(self, v: Self) -> Self {
        Self { c: self.c.saturating_add(v.c) }
    }
}
