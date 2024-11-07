use diesel::{
    sql_types::{BigInt, Text},
    AsExpression, FromSqlRow,
};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper, diesel_string_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::schema_sqlite_types::Integer;

use super::ProfileStateInternal;


#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Eq,
    TryFromPrimitive,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Integer)]
#[repr(i64)]
pub enum ProfileTextModerationState {
    Empty = 0,
    WaitingBotOrHumanModeration = 1,
    WaitingHumanModeration = 2,
    AcceptedByBot = 3,
    AcceptedByHuman = 4,
    RejectedByBot = 5,
    RejectedByHuman = 6,
}

impl ProfileTextModerationState {
    pub fn is_accepted(&self) -> bool {
        match self {
            Self::Empty |
            Self::AcceptedByBot |
            Self::AcceptedByHuman => true,
            Self::WaitingBotOrHumanModeration |
            Self::WaitingHumanModeration |
            Self::RejectedByBot |
            Self::RejectedByHuman => false,
        }
    }
}

impl Default for ProfileTextModerationState {
    fn default() -> Self {
        Self::Empty
    }
}

diesel_i64_try_from!(ProfileTextModerationState);

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

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Text)]
pub struct ProfileTextModerationRejectedReasonDetails {
    value: String,
}

impl ProfileTextModerationRejectedReasonDetails {
    pub fn new(value: String) -> Self {
        Self {
            value
        }
    }

    pub fn into_string(self) -> String {
        self.value
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

diesel_string_wrapper!(ProfileTextModerationRejectedReasonDetails);

#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    ToSchema,
)]
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
