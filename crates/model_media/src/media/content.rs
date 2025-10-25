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
pub struct MediaContentModerationRejectedReasonCategory {
    pub value: i64,
}

impl MediaContentModerationRejectedReasonCategory {
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.value
    }
}

diesel_i64_wrapper!(MediaContentModerationRejectedReasonCategory);

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
pub struct MediaContentModerationRejectedReasonDetails {
    // Non-empty string
    value: NonEmptyString,
}

impl MediaContentModerationRejectedReasonDetails {
    pub fn new(value: NonEmptyString) -> Self {
        Self { value }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

impl TryFrom<NonEmptyString> for MediaContentModerationRejectedReasonDetails {
    type Error = String;

    fn try_from(value: NonEmptyString) -> Result<Self, Self::Error> {
        Ok(Self { value })
    }
}

impl AsRef<str> for MediaContentModerationRejectedReasonDetails {
    fn as_ref(&self) -> &str {
        self.value.as_str()
    }
}

diesel_non_empty_string_wrapper!(MediaContentModerationRejectedReasonDetails);
