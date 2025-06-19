use diesel::{
    AsExpression, FromSqlRow,
    sql_types::{BigInt, Text},
};
use model_server_data::ProfileTextModerationState;
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_wrapper, diesel_string_wrapper};
use utoipa::{IntoParams, ToSchema};

use super::ProfileStateInternal;

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
pub struct ProfileTextModerationRejectedReasonCategory {
    pub value: i64,
}

impl ProfileTextModerationRejectedReasonCategory {
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.value
    }
}

diesel_i64_wrapper!(ProfileTextModerationRejectedReasonCategory);

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
pub struct ProfileTextModerationRejectedReasonDetails {
    value: String,
}

impl ProfileTextModerationRejectedReasonDetails {
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

diesel_string_wrapper!(ProfileTextModerationRejectedReasonDetails);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ProfileTextModerationInfo {
    pub state: ProfileTextModerationState,
    pub rejected_reason_category: Option<ProfileTextModerationRejectedReasonCategory>,
    pub rejected_reason_details: Option<ProfileTextModerationRejectedReasonDetails>,
}

impl From<ProfileStateInternal> for ProfileTextModerationInfo {
    fn from(value: ProfileStateInternal) -> Self {
        Self {
            state: value.profile_text_moderation_state,
            rejected_reason_category: value.profile_text_moderation_rejected_reason_category,
            rejected_reason_details: value.profile_text_moderation_rejected_reason_details,
        }
    }
}
