use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use model::UnixTime;
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Copy, Default, Serialize, FromSqlRow, AsExpression)]
#[diesel(sql_type = BigInt)]
pub struct LastSeenUnixTime {
    pub ut: UnixTime,
}

impl LastSeenUnixTime {
    pub fn current_time() -> Self {
        Self {
            ut: UnixTime::current_time(),
        }
    }
}

impl TryFrom<i64> for LastSeenUnixTime {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self {
            ut: UnixTime::new(value),
        })
    }
}

impl AsRef<i64> for LastSeenUnixTime {
    fn as_ref(&self) -> &i64 {
        self.ut.as_i64()
    }
}

diesel_i64_wrapper!(LastSeenUnixTime);

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

/// Account's most recent disconnect time.
///
/// If the last seen time is not None, then it is Unix timestamp or -1 if
/// the profile is currently online.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct LastSeenTime(i64);

impl LastSeenTime {
    pub const ONLINE: Self = Self(-1);

    pub fn new(raw: i64) -> Self {
        Self(raw)
    }

    pub fn raw(&self) -> i64 {
        self.0
    }

    /// Return None if account is currently online.
    pub fn last_seen_unix_time(&self) -> Option<LastSeenUnixTime> {
        if *self != Self::ONLINE {
            Some(LastSeenUnixTime {
                ut: UnixTime::new(self.raw()),
            })
        } else {
            None
        }
    }
}

impl From<LastSeenUnixTime> for LastSeenTime {
    fn from(value: LastSeenUnixTime) -> Self {
        Self(value.ut.ut)
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
