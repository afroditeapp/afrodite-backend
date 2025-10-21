use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{SimpleDieselEnum, diesel_i64_struct_try_from};
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
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Integer)]
#[repr(i64)]
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
#[diesel(sql_type = Integer)]
#[repr(i64)]
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

#[derive(Debug, Clone, Copy, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Integer)]
pub struct ProfileNameModerationState(pub ProfileStringModerationState);

impl From<ProfileNameModerationState> for i64 {
    fn from(value: ProfileNameModerationState) -> Self {
        value.0 as i64
    }
}

impl TryFrom<i64> for ProfileNameModerationState {
    type Error = <ProfileStringModerationState as TryFrom<i64>>::Error;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        TryFrom::<i64>::try_from(value).map(Self)
    }
}

diesel_i64_struct_try_from!(ProfileNameModerationState);

impl From<ProfileStringModerationState> for ProfileNameModerationState {
    fn from(value: ProfileStringModerationState) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Integer)]
pub struct ProfileTextModerationState(pub ProfileStringModerationState);

impl From<ProfileTextModerationState> for i64 {
    fn from(value: ProfileTextModerationState) -> Self {
        value.0 as i64
    }
}

impl TryFrom<i64> for ProfileTextModerationState {
    type Error = <ProfileStringModerationState as TryFrom<i64>>::Error;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        TryFrom::<i64>::try_from(value).map(Self)
    }
}

diesel_i64_struct_try_from!(ProfileTextModerationState);

impl From<ProfileStringModerationState> for ProfileTextModerationState {
    fn from(value: ProfileStringModerationState) -> Self {
        Self(value)
    }
}
