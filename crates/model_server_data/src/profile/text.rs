use diesel::{deserialize::FromSqlRow, expression::AsExpression};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_struct_try_from, diesel_i64_try_from};
use unicode_segmentation::UnicodeSegmentation;
use utoipa::ToSchema;

use crate::schema_sqlite_types::Integer;

use super::ProfileInternal;

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
    /// Profile text is empty
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
            Self::Empty | Self::AcceptedByBot | Self::AcceptedByHuman => true,
            Self::WaitingBotOrHumanModeration
            | Self::WaitingHumanModeration
            | Self::RejectedByBot
            | Self::RejectedByHuman => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::Empty
    }
}

impl Default for ProfileTextModerationState {
    fn default() -> Self {
        Self::Empty
    }
}

diesel_i64_try_from!(ProfileTextModerationState);

#[derive(Debug, Clone, Copy)]
pub struct ProfileTextCharacterCount {
    count: u16,
}

impl ProfileTextCharacterCount {
    pub fn new(data: &ProfileInternal) -> Self {
        Self {
            count: data
                .profile_text
                .graphemes(true)
                .count()
                .try_into()
                .unwrap_or(u16::MAX),
        }
    }
}

/// Filter value for profile text min characters.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Eq,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = Integer)]
pub struct ProfileTextMinCharactersFilter {
    pub value: u16,
}

impl ProfileTextMinCharactersFilter {
    pub fn is_match(&self, count: ProfileTextCharacterCount) -> bool {
        count.count >= self.value
    }
}

impl TryFrom<i64> for ProfileTextMinCharactersFilter {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.clamp(0, u16::MAX.into()) as u16
        })
    }
}

impl From<ProfileTextMinCharactersFilter> for i64 {
    fn from(value: ProfileTextMinCharactersFilter) -> Self {
        value.value as i64
    }
}

diesel_i64_struct_try_from!(ProfileTextMinCharactersFilter);

/// Filter value for profile text max characters.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Eq,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = Integer)]
pub struct ProfileTextMaxCharactersFilter {
    pub value: u16,
}

impl ProfileTextMaxCharactersFilter {
    pub fn is_match(&self, count: ProfileTextCharacterCount) -> bool {
        count.count <= self.value
    }
}

impl TryFrom<i64> for ProfileTextMaxCharactersFilter {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.clamp(0, u16::MAX.into()) as u16
        })
    }
}

impl From<ProfileTextMaxCharactersFilter> for i64 {
    fn from(value: ProfileTextMaxCharactersFilter) -> Self {
        value.value as i64
    }
}

diesel_i64_struct_try_from!(ProfileTextMaxCharactersFilter);
