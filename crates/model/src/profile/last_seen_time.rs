
use diesel::{
    sql_types::BigInt,
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_wrapper, UnixTime};
use utoipa::{IntoParams, ToSchema};


/// Account's most recent disconnect time.
///
/// If the last seen time is not None, then it is Unix timestamp or -1 if
/// the profile is currently online.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct LastSeenTime(pub(crate) i64);

impl LastSeenTime {
    pub const ONLINE: Self = Self(-1);
    pub(crate) const MIN_VALUE: i64 = Self::ONLINE.0;
}

impl From<UnixTime> for LastSeenTime {
    fn from(value: UnixTime) -> Self {
        Self(value.unix_time)
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
#[serde(transparent)]
pub struct LastSeenTimeFilter {
    pub value: i64,
}

impl LastSeenTimeFilter {
    const ONLINE: Self = Self { value: -1 };
    pub(crate) const MIN_VALUE: i64 = -1;

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
        } else if last_seen_time.0 <= current_time.unix_time {
            let seconds_since_last_seen = last_seen_time.0.abs_diff(current_time.unix_time);
            let max_seconds_since = self.value as u64;
            seconds_since_last_seen <= max_seconds_since
        } else {
            false
        }
    }
}

diesel_i64_wrapper!(LastSeenTimeFilter);
