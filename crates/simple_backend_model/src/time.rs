use diesel::{sql_types::BigInt, AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};
use simple_backend_utils::current_unix_time;
use utoipa::{ToSchema, IntoParams};

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
    Default,
    sqlx::Type,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct UnixTime {
    pub unix_time: i64,
}

impl UnixTime {
    pub fn new(value: i64) -> Self {
        Self { unix_time: value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.unix_time
    }

    pub fn current_time() -> Self {
        Self {
            unix_time: current_unix_time(),
        }
    }
}

diesel_i64_wrapper!(UnixTime);
