use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::{IntoParams, ToSchema};

use super::sync_version_wrappers;

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

/// Version number for asymmetric encryption public key data which
/// client defines. This allows changing client's end-to-end crypto
/// implementation.
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
pub struct PublicKeyVersion {
    pub version: i64,
}

impl PublicKeyVersion {
    pub fn new(id: i64) -> Self {
        Self { version: id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.version
    }
}

diesel_i64_wrapper!(PublicKeyVersion);

#[derive(
    Debug,
    Clone,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
)]
pub struct PublicKeyIdAndVersion {
    pub id: PublicKeyId,
    pub version: PublicKeyVersion,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct NewReceivedLikesCountResult {
    pub v: ReceivedLikesSyncVersion,
    pub c: NewReceivedLikesCount,
}

sync_version_wrappers!(
    ReceivedBlocksSyncVersion,
    /// Sync version for new received likes count
    ReceivedLikesSyncVersion,
    SentBlocksSyncVersion,
    SentLikesSyncVersion,
    MatchesSyncVersion,
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
        Self { c: self.c.saturating_add(1) }
    }

    /// Return new decremented value using `max(0, value - 1)`.
    pub fn decrement(&self) -> Self {
        Self { c: i64::max(0, self.c - 1) }
    }
}

diesel_i64_wrapper!(NewReceivedLikesCount);
