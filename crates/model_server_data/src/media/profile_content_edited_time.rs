use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::{UnixTime, diesel_i64_wrapper};
use utoipa::ToSchema;

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Serialize,
    Deserialize,
    ToSchema,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct ProfileContentEditedTime(UnixTime);

impl ProfileContentEditedTime {
    pub fn new(value: i64) -> Self {
        Self(UnixTime::new(value))
    }

    pub fn as_i64(&self) -> &i64 {
        &self.0.ut
    }

    pub fn current_time() -> Self {
        Self(UnixTime::current_time())
    }
}

diesel_i64_wrapper!(ProfileContentEditedTime);
