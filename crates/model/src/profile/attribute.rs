use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::SmallInt};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{SimpleDieselEnum, diesel_i16_wrapper};
use utoipa::ToSchema;

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    ToSchema,
    TryFromPrimitive,
    SimpleDieselEnum,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
pub enum AttributeOrderMode {
    #[default]
    OrderNumber = 0,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct PartialProfileAttributesConfig {
    pub attribute_order: AttributeOrderMode,
    pub attributes: Vec<ProfileAttributeInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileAttributeInfo {
    pub id: AttributeId,
    pub h: AttributeHash,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AttributeHash {
    h: String,
}

impl AttributeHash {
    pub fn new(h: String) -> Self {
        Self { h }
    }

    pub fn as_str(&self) -> &str {
        &self.h
    }
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
    PartialOrd,
    Ord,
    Hash,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = SmallInt)]
pub struct AttributeId(#[serde(deserialize_with = "deserialize_non_negative_i16")] i16);

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

impl AttributeId {
    /// The `id` must be 0 or greater.
    pub fn new(id: i16) -> Self {
        assert!(id >= 0);
        Self(id)
    }

    pub fn to_i16(&self) -> i16 {
        self.0
    }

    pub fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

impl TryFrom<i16> for AttributeId {
    type Error = String;
    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl AsRef<i16> for AttributeId {
    fn as_ref(&self) -> &i16 {
        &self.0
    }
}

diesel_i16_wrapper!(AttributeId);
