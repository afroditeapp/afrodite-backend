use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_try_from;
use utoipa::ToSchema;

use crate::schema_sqlite_types::Integer;

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

    pub fn is_moderated(&self) -> bool {
        match self {
            Self::AcceptedByBot |
            Self::AcceptedByHuman |
            Self::RejectedByBot |
            Self::RejectedByHuman => true,
            Self::Empty |
            Self::WaitingBotOrHumanModeration |
            Self::WaitingHumanModeration => false,
        }
    }
}

impl Default for ProfileTextModerationState {
    fn default() -> Self {
        Self::Empty
    }
}

diesel_i64_try_from!(ProfileTextModerationState);
