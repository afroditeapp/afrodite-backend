use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_struct_try_from;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum AttributeOrderMode {
    OrderNumber,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
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
#[diesel(sql_type = BigInt)]
pub struct AttributeId(u16);

impl AttributeId {
    pub fn new(id: u16) -> Self {
        Self(id)
    }

    pub fn to_usize(&self) -> usize {
        self.0.into()
    }
}

impl TryFrom<i64> for AttributeId {
    type Error = String;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value: u16 = value
            .try_into()
            .map_err(|e: std::num::TryFromIntError| e.to_string())?;
        Ok(Self(value))
    }
}

impl From<AttributeId> for i64 {
    fn from(value: AttributeId) -> Self {
        value.0.into()
    }
}

diesel_i64_struct_try_from!(AttributeId);
