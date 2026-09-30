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
    WaitingAdminBot = 0,
    WaitingAdmin = 1,
    AcceptedByAdminBot = 2,
    AcceptedByAdmin = 3,
    AcceptedByAllowlist = 4,
    RejectedByAdminBot = 5,
    RejectedByAdmin = 6,
}

impl ProfileStringModerationState {
    pub fn is_accepted(&self) -> bool {
        match self {
            Self::AcceptedByAdminBot | Self::AcceptedByAdmin | Self::AcceptedByAllowlist => true,
            Self::WaitingAdminBot
            | Self::WaitingAdmin
            | Self::RejectedByAdminBot
            | Self::RejectedByAdmin => false,
        }
    }

    pub fn is_rejected(&self) -> bool {
        match self {
            Self::RejectedByAdminBot | Self::RejectedByAdmin => true,
            Self::WaitingAdminBot
            | Self::WaitingAdmin
            | Self::AcceptedByAdminBot
            | Self::AcceptedByAdmin
            | Self::AcceptedByAllowlist => false,
        }
    }

    /// Returns `true` if the moderation state changed visually for the user
    /// when transitioning to `new_state`.
    pub fn send_notification_to_user(&self, new_state: Self) -> bool {
        self.is_accepted() != new_state.is_accepted()
            || self.is_rejected() != new_state.is_rejected()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProfileNameModerationState(pub ProfileStringModerationState);

#[derive(Debug, Clone, Copy)]
pub struct ProfileTextModerationState(pub ProfileStringModerationState);
