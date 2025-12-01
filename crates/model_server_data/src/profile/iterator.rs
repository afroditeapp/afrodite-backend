use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::SmallInt};
use model::{AccountId, LastSeenTime, NextNumberStorage, ProfileContentVersion};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i16_wrapper;
use utoipa::{IntoParams, ToSchema};

use super::ProfileVersion;

/// Session ID type for profile iterator so that client can detect
/// server restarts and ask user to refresh profiles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProfileIteratorSessionIdInternal {
    id: i64,
}

impl ProfileIteratorSessionIdInternal {
    /// Current implementation uses i64. Only requirement for this
    /// type is that next one should be different than the previous.
    pub fn create(storage: &mut NextNumberStorage) -> Self {
        Self {
            id: storage.get_and_increment(),
        }
    }
}

/// Session ID type for profile iterator so that client can detect
/// server restarts and ask user to refresh profiles.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ProfileIteratorSessionId {
    id: i64,
}

impl From<ProfileIteratorSessionIdInternal> for ProfileIteratorSessionId {
    fn from(value: ProfileIteratorSessionIdInternal) -> Self {
        Self { id: value.id }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ProfileLink {
    a: AccountId,
    p: ProfileVersion,
    c: ProfileContentVersion,
    l: Option<LastSeenTime>,
}

impl ProfileLink {
    pub fn new(
        id: AccountId,
        version: ProfileVersion,
        content_version: ProfileContentVersion,
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

/// Profile iterator min distance in kilometers.
///
/// The value is equal or greater than 1.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    IntoParams,
    PartialEq,
    Eq,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = SmallInt)]
pub struct MinDistanceKm {
    pub value: i16,
}

impl TryFrom<i16> for MinDistanceKm {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Ok(Self { value })
    }
}

impl AsRef<i16> for MinDistanceKm {
    fn as_ref(&self) -> &i16 {
        &self.value
    }
}

diesel_i16_wrapper!(MinDistanceKm);

/// Profile iterator max distance in kilometers.
///
/// The value is equal or greater than 1.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    IntoParams,
    PartialEq,
    Eq,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = SmallInt)]
pub struct MaxDistanceKm {
    pub value: i16,
}

impl TryFrom<i16> for MaxDistanceKm {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Ok(Self { value })
    }
}

impl AsRef<i16> for MaxDistanceKm {
    fn as_ref(&self) -> &i16 {
        &self.value
    }
}

diesel_i16_wrapper!(MaxDistanceKm);
