use diesel::sql_types::SmallInt;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::SimpleDieselEnum;
use utoipa::ToSchema;

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
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
pub enum ProfileStringModerationContentType {
    ProfileName = 0,
    ProfileText = 1,
}

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
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
pub enum ProfileStringModerationState {
    WaitingBotOrHumanModeration = 0,
    WaitingHumanModeration = 1,
    AcceptedByBot = 2,
    AcceptedByHuman = 3,
    AcceptedByAllowlist = 4,
    RejectedByBot = 5,
    RejectedByHuman = 6,
}

impl ProfileStringModerationState {
    pub fn is_accepted(&self) -> bool {
        match self {
            Self::AcceptedByBot | Self::AcceptedByHuman | Self::AcceptedByAllowlist => true,
            Self::WaitingBotOrHumanModeration
            | Self::WaitingHumanModeration
            | Self::RejectedByBot
            | Self::RejectedByHuman => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProfileNameModerationState(pub ProfileStringModerationState);

#[derive(Debug, Clone, Copy)]
pub struct ProfileTextModerationState(pub ProfileStringModerationState);
