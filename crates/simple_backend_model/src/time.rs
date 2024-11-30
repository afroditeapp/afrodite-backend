use chrono::Datelike;
use diesel::{sql_types::BigInt, AsExpression, FromSqlRow};
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
        chrono::DateTime::from_timestamp(self.ut, 0).map(|v| v.year())
    }

    /// Return decremented time value (self.ut - 1). Implemented using
    /// `saturating_sub`.
    pub fn decrement(self) -> Self {
        Self {
            ut: self.ut.saturating_sub(1),
        }
    }
}

diesel_i64_wrapper!(UnixTime);
