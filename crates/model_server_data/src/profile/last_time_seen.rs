use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use model::{LastSeenTime, LastSeenUnixTime, UnixTime};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Copy, Default, Serialize, FromSqlRow, AsExpression)]
#[diesel(sql_type = BigInt)]
pub struct AutomaticProfileSearchLastSeenUnixTime {
    pub ut: UnixTime,
}

impl TryFrom<i64> for AutomaticProfileSearchLastSeenUnixTime {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self {
            ut: UnixTime::new(value),
        })
    }
}

impl AsRef<i64> for AutomaticProfileSearchLastSeenUnixTime {
    fn as_ref(&self) -> &i64 {
        self.ut.as_i64()
    }
}

diesel_i64_wrapper!(AutomaticProfileSearchLastSeenUnixTime);

impl From<LastSeenUnixTime> for AutomaticProfileSearchLastSeenUnixTime {
    fn from(value: LastSeenUnixTime) -> Self {
        Self { ut: value.ut }
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

    pub fn is_match(&self, last_seen_time: LastSeenTime, current_time: &UnixTime) -> bool {
        if *self == Self::ONLINE {
            last_seen_time == LastSeenTime::ONLINE
        } else if last_seen_time.raw() <= current_time.ut {
            let seconds_since_last_seen = last_seen_time.raw().abs_diff(current_time.ut);
            let max_seconds_since = self.value as u64;
            last_seen_time == LastSeenTime::ONLINE || seconds_since_last_seen <= max_seconds_since
        } else {
            false
        }
    }
}

impl TryFrom<i64> for LastSeenTimeFilter {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self { value })
    }
}

impl AsRef<i64> for LastSeenTimeFilter {
    fn as_ref(&self) -> &i64 {
        &self.value
    }
}

diesel_i64_wrapper!(LastSeenTimeFilter);
