use diesel::{
    sql_types::{BigInt, Text},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_wrapper, diesel_string_wrapper};
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
pub struct ProfileContentModerationRejectedReasonCategory {
    pub value: i64,
}

impl ProfileContentModerationRejectedReasonCategory {
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.value
    }
}

diesel_i64_wrapper!(ProfileContentModerationRejectedReasonCategory);

#[derive(
    Debug,
    Deserialize,
    Serialize,
    ToSchema,
    Clone,
    Eq,
    Hash,
    PartialEq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Text)]
pub struct ProfileContentModerationRejectedReasonDetails {
    value: String,
}

impl ProfileContentModerationRejectedReasonDetails {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn into_string(self) -> String {
        self.value
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

diesel_string_wrapper!(ProfileContentModerationRejectedReasonDetails);
