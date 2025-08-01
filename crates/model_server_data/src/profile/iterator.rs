use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use model::{AccountId, NextNumberStorage, ProfileContentVersion};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::{IntoParams, ToSchema};

use super::{LastSeenTime, ProfileVersion};

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
    /// This is optional because media component owns it.
    c: Option<ProfileContentVersion>,
    /// If the last seen time is not None, then it is Unix timestamp or -1 if
    /// the profile is currently online.
    l: Option<LastSeenTime>,
}

impl ProfileLink {
    pub fn new(
        id: AccountId,
        version: ProfileVersion,
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

    pub fn last_seen_time(&self) -> Option<LastSeenTime> {
        self.l
    }

    pub fn set_last_seen_time(&mut self, value: LastSeenTime) {
        self.l = Some(value);
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
#[diesel(sql_type = BigInt)]
pub struct MinDistanceKm {
    pub value: i64,
}

impl MinDistanceKm {
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.value
    }
}

diesel_i64_wrapper!(MinDistanceKm);

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
#[diesel(sql_type = BigInt)]
pub struct MaxDistanceKm {
    pub value: i64,
}

impl MaxDistanceKm {
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.value
    }
}

diesel_i64_wrapper!(MaxDistanceKm);
