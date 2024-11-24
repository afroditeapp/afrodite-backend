use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::ToSchema;

use crate::{AccountId, NextNumberStorage};

use crate::MatchesSyncVersion;

/// Session ID type for matches iterator so that client can detect
/// server restarts and ask user to refresh matches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MatchesIteratorSessionIdInternal {
    id: i64,
}

impl MatchesIteratorSessionIdInternal {
    /// Current implementation uses i64. Only requirement for this
    /// type is that next one should be different than the previous.
    pub fn create(storage: &mut NextNumberStorage) -> Self {
        Self {
            id: storage.get_and_increment(),
        }
    }
}

/// Session ID type for matches iterator so that client can detect
/// server restarts and ask user to matches.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct MatchesIteratorSessionId {
    id: i64,
}

impl From<MatchesIteratorSessionIdInternal> for MatchesIteratorSessionId {
    fn from(value: MatchesIteratorSessionIdInternal) -> Self {
        Self {
            id: value.id,
        }
    }
}

impl From<MatchesIteratorSessionId> for MatchesIteratorSessionIdInternal {
    fn from(value: MatchesIteratorSessionId) -> Self {
        Self {
            id: value.id,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct AllMatchesPage {
    /// This version can be sent to the server when WebSocket protocol
    /// data sync is happening.
    pub version: MatchesSyncVersion,
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResetMatchesIteratorResult {
    pub s: MatchesIteratorSessionId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct MatchesPage {
    pub p: Vec<AccountId>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_invalid_iterator_session_id: bool,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Deserialize,
    Serialize,
    PartialEq,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct MatchId {
    pub id: i64,
}

impl MatchId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }

    /// Return new incremented value using `saturated_add`.
    pub fn increment(&self) -> Self {
        Self { id: self.id.saturating_add(1) }
    }

    /// This returns -1 if ID is not incremented.
    pub fn next_id_to_latest_used_id(&self) -> Self {
        Self { id: self.id - 1 }
    }
}

diesel_i64_wrapper!(MatchId);

impl From<MatchId> for i64 {
    fn from(value: MatchId) -> Self {
        value.id
    }
}
