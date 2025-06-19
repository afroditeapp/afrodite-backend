use chrono::{Datelike, Timelike};
use diesel::{AsExpression, FromSqlRow, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_utils::current_unix_time;
use utoipa::{IntoParams, ToSchema};

use crate::macros::diesel_i64_wrapper;

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
pub struct UnixTime {
    pub ut: i64,
}

impl UnixTime {
    pub fn new(value: i64) -> Self {
        Self { ut: value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.ut
    }

    pub fn current_time() -> Self {
        Self {
            ut: current_unix_time(),
        }
    }

    pub fn year(&self) -> Option<i32> {
        self.to_chrono_time().map(|v| v.year())
    }

    pub fn hour(&self) -> Option<u32> {
        self.to_chrono_time().map(|v| v.hour())
    }

    /// Return decremented time value (self.ut - 1). Implemented using
    /// `saturating_sub`.
    pub fn decrement(self) -> Self {
        Self {
            ut: self.ut.saturating_sub(1),
        }
    }

    pub fn add_seconds(&self, seconds: u32) -> Self {
        let seconds: i64 = seconds.into();
        Self {
            ut: self.ut + seconds,
        }
    }

    pub fn to_chrono_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::from_timestamp(self.ut, 0)
    }
}

diesel_i64_wrapper!(UnixTime);

impl From<chrono::DateTime<chrono::Utc>> for UnixTime {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            ut: value.timestamp(),
        }
    }
}
