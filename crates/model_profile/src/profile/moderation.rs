use diesel::{
    AsExpression, FromSqlRow, Selectable,
    prelude::Queryable,
    sql_types::{BigInt, Text},
};
use model_server_data::ProfileStringModerationState;
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
pub struct ProfileModerationRejectedReasonCategory {
    pub value: i64,
}

impl ProfileModerationRejectedReasonCategory {
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.value
    }
}

diesel_i64_wrapper!(ProfileModerationRejectedReasonCategory);

/// Text might be empty.
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
pub struct ProfileModerationRejectedReasonDetails {
    value: String,
}

impl ProfileModerationRejectedReasonDetails {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn into_string(self) -> String {
        self.value
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

diesel_string_wrapper!(ProfileModerationRejectedReasonDetails);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Selectable, Queryable)]
#[diesel(table_name = crate::schema::profile_moderation)]
#[diesel(check_for_backend(crate::Db))]
pub struct ProfileModerationInfo {
    #[diesel(column_name = "state_type")]
    pub state: ProfileStringModerationState,
    pub rejected_reason_category: Option<ProfileModerationRejectedReasonCategory>,
    pub rejected_reason_details: ProfileModerationRejectedReasonDetails,
}
