
use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use model::UnixTime;
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::{IntoParams, ToSchema};

/// Account's most recent disconnect time.
///
/// If the last seen time is not None, then it is Unix timestamp or -1 if
/// the profile is currently online.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct LastSeenTime(i64);

impl LastSeenTime {
    pub const ONLINE: Self = Self(-1);
    pub const MIN_VALUE: i64 = Self::ONLINE.0;

    pub fn new(raw: i64) -> Self {
        Self(raw)
    }

    pub fn raw(&self) -> i64 {
        self.0
    }
}

impl From<UnixTime> for LastSeenTime {
    fn from(value: UnixTime) -> Self {
        Self(value.ut)
    }
}

/// Filter value for last seen time.
///
/// Possible values:
/// - Value -1 is show only profiles which are online.
/// - Zero and positive values are max seconds since the profile has been online.
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
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct LastSeenTimeFilter {
    pub value: i64,
}

impl LastSeenTimeFilter {
    const ONLINE: Self = Self { value: -1 };
    pub const MIN_VALUE: i64 = -1;

    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.value
    }

    pub fn is_match(
        &self,
        last_seen_time: LastSeenTime,
        current_time: &UnixTime,
    ) -> bool {
        if *self == Self::ONLINE {
            last_seen_time == LastSeenTime::ONLINE
        } else if last_seen_time.raw() <= current_time.ut {
            let seconds_since_last_seen = last_seen_time.raw().abs_diff(current_time.ut);
            let max_seconds_since = self.value as u64;
            last_seen_time == LastSeenTime::ONLINE ||
            seconds_since_last_seen <= max_seconds_since
        } else {
            false
        }
    }
}

diesel_i64_wrapper!(LastSeenTimeFilter);
