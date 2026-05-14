use diesel::{AsExpression, FromSqlRow, sql_types::SmallInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::{SimpleDieselEnum, diesel_i16_wrapper};
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
    Hash,
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
    num_enum::TryFromPrimitive,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
pub enum VerificationMethod {
    DebugAccept = 0,
    DebugReject = 1,
    Eudi = 2,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct AccountVerificationErrorFlags: i16 {
        const VERIFICATION_DATA_PARSING_FAILED = 0x1;
        const VERIFICATION_DATA_VERIFICATION_FAILED = 0x2;
        const PROFILE_AGE_RANGE_VERIFICATION_FAILED = 0x4;
        const PROFILE_AGE_RANGE_MISMATCH = 0x8;
        const PROFILE_NAME_VERIFICATION_FAILED = 0x10;
        const PROFILE_NAME_MISMATCH = 0x20;
        const SECURITY_CONTENT_VERIFICATION_FAILED = 0x40;
        const SECURITY_CONTENT_MISMATCH = 0x80;
    }
}

impl TryFrom<i16> for AccountVerificationErrorFlags {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::from_bits(value).ok_or_else(|| "Unknown bitflag".to_string())
    }
}

/// Value for account verification error flags.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Eq,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = SmallInt)]
pub struct AccountVerificationErrorFlagsValue {
    #[serde(deserialize_with = "deserialize_account_verification_error_flags_i16")]
    pub v: i16,
}

fn deserialize_account_verification_error_flags_i16<'de, D>(
    deserializer: D,
) -> Result<i16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = i16::deserialize(deserializer)?;
    AccountVerificationErrorFlags::try_from(v)
        .map(|_| v)
        .map_err(serde::de::Error::custom)
}

impl TryFrom<i16> for AccountVerificationErrorFlagsValue {
    type Error = String;

    fn try_from(v: i16) -> Result<Self, Self::Error> {
        AccountVerificationErrorFlags::try_from(v)?;
        Ok(Self { v })
    }
}

impl AsRef<i16> for AccountVerificationErrorFlagsValue {
    fn as_ref(&self) -> &i16 {
        &self.v
    }
}

impl From<AccountVerificationErrorFlags> for AccountVerificationErrorFlagsValue {
    fn from(value: AccountVerificationErrorFlags) -> Self {
        Self { v: value.bits() }
    }
}

diesel_i16_wrapper!(AccountVerificationErrorFlagsValue);

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountVerificationQueueItem {
    pub verification_method: VerificationMethod,
    pub verification_data: String,
    pub verification_scope: AccountVerificationScope,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct AccountVerificationScope {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub security_content: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_age_range: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_name: bool,
}
