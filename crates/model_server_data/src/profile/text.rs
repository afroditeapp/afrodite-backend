use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::SmallInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i16_wrapper;
use unicode_segmentation::UnicodeSegmentation;
use utoipa::ToSchema;

use super::ProfileInternal;

#[derive(Debug, Clone, Copy)]
pub struct ProfileTextCharacterCount {
    count: u16,
}

impl ProfileTextCharacterCount {
    pub fn new(data: &ProfileInternal) -> Self {
        Self {
            count: data
                .profile_text
                .as_ref()
                .map(|t| t.as_str())
                .unwrap_or_default()
                .graphemes(true)
                .count()
                .try_into()
                .unwrap_or(u16::MAX),
        }
    }
}

/// Filter value for profile text min characters.
/// The value must be 0 or greater.
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
#[diesel(sql_type = SmallInt)]
pub struct ProfileTextMinCharactersFilter {
    #[serde(deserialize_with = "deserialize_non_negative_i16")]
    value: i16,
}

fn deserialize_non_negative_i16<'de, D>(deserializer: D) -> Result<i16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = i16::deserialize(deserializer)?;
    if v < 0 {
        Err(serde::de::Error::custom("negative value not allowed"))
    } else {
        Ok(v)
    }
}

impl ProfileTextMinCharactersFilter {
    pub fn is_match(&self, count: ProfileTextCharacterCount) -> bool {
        count.count >= self.value as u16
    }
}

impl TryFrom<i16> for ProfileTextMinCharactersFilter {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Ok(Self { value })
    }
}

impl AsRef<i16> for ProfileTextMinCharactersFilter {
    fn as_ref(&self) -> &i16 {
        &self.value
    }
}

diesel_i16_wrapper!(ProfileTextMinCharactersFilter);

/// Filter value for profile text max characters.
/// The value must be 0 or greater.
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
#[diesel(sql_type = SmallInt)]
pub struct ProfileTextMaxCharactersFilter {
    #[serde(deserialize_with = "deserialize_non_negative_i16")]
    value: i16,
}

impl ProfileTextMaxCharactersFilter {
    pub fn is_match(&self, count: ProfileTextCharacterCount) -> bool {
        count.count <= self.value as u16
    }
}

impl TryFrom<i16> for ProfileTextMaxCharactersFilter {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Ok(Self { value })
    }
}

impl AsRef<i16> for ProfileTextMaxCharactersFilter {
    fn as_ref(&self) -> &i16 {
        &self.value
    }
}

diesel_i16_wrapper!(ProfileTextMaxCharactersFilter);
