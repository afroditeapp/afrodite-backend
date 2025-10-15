use diesel::{
    AsExpression, FromSqlRow,
    sql_types::{BigInt, Text},
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{NonEmptyString, diesel_i64_wrapper, diesel_non_empty_string_wrapper};
use utoipa::{IntoParams, ToSchema};

#[derive(
    Debug,
    Serialize,
    Deserialize,
    ToSchema,
    Clone,
    Eq,
    Hash,
    PartialEq,
    IntoParams,
    Copy,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct AccountBanReasonCategory {
    pub value: i64,
}

impl AccountBanReasonCategory {
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.value
    }
}

diesel_i64_wrapper!(AccountBanReasonCategory);

#[derive(
    Debug,
    Clone,
    Deserialize,
    Serialize,
    ToSchema,
    Eq,
    Hash,
    PartialEq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Text)]
pub struct AccountBanReasonDetails {
    // Non-empty string
    value: NonEmptyString,
}

impl AccountBanReasonDetails {
    pub fn new(value: NonEmptyString) -> Self {
        Self { value }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

diesel_non_empty_string_wrapper!(AccountBanReasonDetails);
