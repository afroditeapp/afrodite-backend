use diesel::{
    AsExpression, FromSqlRow, Selectable,
    prelude::Queryable,
    sql_types::{SmallInt, Text},
};
use model::UnixTime;
use model_server_data::ProfileStringModerationState;
use serde::{Deserialize, Serialize};
use simple_backend_model::{NonEmptyString, diesel_i16_wrapper, diesel_non_empty_string_wrapper};
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
#[diesel(sql_type = SmallInt)]
pub struct ProfileStringModerationRejectedReasonCategory {
    pub value: i16,
}

impl TryFrom<i16> for ProfileStringModerationRejectedReasonCategory {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Ok(Self { value })
    }
}

impl AsRef<i16> for ProfileStringModerationRejectedReasonCategory {
    fn as_ref(&self) -> &i16 {
        &self.value
    }
}

diesel_i16_wrapper!(ProfileStringModerationRejectedReasonCategory);

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
pub struct ProfileStringModerationRejectedReasonDetails {
    // Non-empty string
    value: NonEmptyString,
}

impl ProfileStringModerationRejectedReasonDetails {
    pub fn new(value: NonEmptyString) -> Self {
        Self { value }
    }

    pub fn reported() -> Self {
        Self {
            value: NonEmptyString::from_string("Reported".to_string()).unwrap(),
        }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

impl TryFrom<NonEmptyString> for ProfileStringModerationRejectedReasonDetails {
    type Error = String;

    fn try_from(value: NonEmptyString) -> Result<Self, Self::Error> {
        Ok(Self { value })
    }
}

impl AsRef<str> for ProfileStringModerationRejectedReasonDetails {
    fn as_ref(&self) -> &str {
        self.value.as_str()
    }
}

diesel_non_empty_string_wrapper!(ProfileStringModerationRejectedReasonDetails);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Selectable, Queryable)]
#[diesel(table_name = crate::schema::profile_moderation)]
#[diesel(check_for_backend(crate::Db))]
pub struct ProfileStringModerationInfo {
    #[diesel(column_name = "state_type")]
    pub state: ProfileStringModerationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_reason_category: Option<ProfileStringModerationRejectedReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_reason_details: Option<ProfileStringModerationRejectedReasonDetails>,
}

/// Only data export uses this.
#[derive(Serialize)]
pub struct ProfileStringModerationCreated {
    pub profile_name: Option<UnixTime>,
    pub profile_text: Option<UnixTime>,
}
