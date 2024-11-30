use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use model::NextNumberStorage;
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::{IntoParams, ToSchema};

/// Session ID type for news iterator so that client can detect
/// server restarts and ask user to refresh news.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NewsIteratorSessionIdInternal {
    pub id: i64,
}

impl NewsIteratorSessionIdInternal {
    /// Current implementation uses i64. Only requirement for this
    /// type is that next one should be different than the previous.
    pub fn create(storage: &mut NextNumberStorage) -> Self {
        Self {
            id: storage.get_and_increment(),
        }
    }
}

/// Publication ID
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    IntoParams,
    PartialEq,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct PublicationId {
    pub id: i64,
}

impl PublicationId {
    /// The value is the same as [crate::MatchId::next_id_to_latest_used_id]
    /// returns if there is no items.
    pub const NO_PUBLICATION_ID: PublicationId = PublicationId { id: -1 };

    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }

    /// Might return -1 if no IDs are used
    pub fn to_latest_used_id(&self) -> Self {
        Self { id: self.id - 1 }
    }
}

diesel_i64_wrapper!(PublicationId);

impl From<PublicationId> for i64 {
    fn from(value: PublicationId) -> Self {
        value.id
    }
}

/// Session ID type for news iterator so that client can detect
/// server restarts and ask user to refresh news.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct NewsIteratorSessionId {
    id: i64,
}

impl From<NewsIteratorSessionIdInternal> for NewsIteratorSessionId {
    fn from(value: NewsIteratorSessionIdInternal) -> Self {
        Self { id: value.id }
    }
}

impl From<NewsIteratorSessionId> for NewsIteratorSessionIdInternal {
    fn from(value: NewsIteratorSessionId) -> Self {
        Self { id: value.id }
    }
}
