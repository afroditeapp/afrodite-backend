use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::{IntoParams, ToSchema};

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

    /// Might return -1 if no IDs are used
    pub fn to_latest_used_id(&self) -> Self {
        Self { id: self.id - 1 }
    }
}

impl TryFrom<i64> for PublicationId {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self { id: value })
    }
}

impl AsRef<i64> for PublicationId {
    fn as_ref(&self) -> &i64 {
        &self.id
    }
}

diesel_i64_wrapper!(PublicationId);

impl From<PublicationId> for i64 {
    fn from(value: PublicationId) -> Self {
        value.id
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct NewsIteratorState {
    pub id_at_reset: PublicationId,
    pub page: i64,
}
