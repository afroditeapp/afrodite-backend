use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_struct_try_from, diesel_i64_try_from};
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
pub enum ProfileModerationContentType {
    ProfileName = 0,
    ProfileText = 1,
}

diesel_i64_try_from!(ProfileModerationContentType);

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
pub enum ProfileModerationState {
    WaitingBotOrHumanModeration = 0,
    WaitingHumanModeration = 1,
    AcceptedByBot = 2,
    AcceptedByHuman = 3,
    AcceptedByAllowlist = 4,
    RejectedByBot = 5,
    RejectedByHuman = 6,
}

impl ProfileModerationState {
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

diesel_i64_try_from!(ProfileModerationState);

#[derive(Debug, Clone, Copy, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Integer)]
pub struct ProfileNameModerationState(pub ProfileModerationState);

impl From<ProfileNameModerationState> for i64 {
    fn from(value: ProfileNameModerationState) -> Self {
        value.0 as i64
    }
}

impl TryFrom<i64> for ProfileNameModerationState {
    type Error = <ProfileModerationState as TryFrom<i64>>::Error;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        TryFrom::<i64>::try_from(value).map(Self)
    }
}

diesel_i64_struct_try_from!(ProfileNameModerationState);

impl From<ProfileModerationState> for ProfileNameModerationState {
    fn from(value: ProfileModerationState) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Integer)]
pub struct ProfileTextModerationState(pub ProfileModerationState);

impl From<ProfileTextModerationState> for i64 {
    fn from(value: ProfileTextModerationState) -> Self {
        value.0 as i64
    }
}

impl TryFrom<i64> for ProfileTextModerationState {
    type Error = <ProfileModerationState as TryFrom<i64>>::Error;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        TryFrom::<i64>::try_from(value).map(Self)
    }
}

diesel_i64_struct_try_from!(ProfileTextModerationState);

impl From<ProfileModerationState> for ProfileTextModerationState {
    fn from(value: ProfileModerationState) -> Self {
        Self(value)
    }
}
