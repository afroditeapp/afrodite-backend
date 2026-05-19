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
    /// Data: "accept" or "reject"
    Debug = 0,
    Eudi = 1,
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
    Hash,
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
    num_enum::TryFromPrimitive,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
pub enum AgeVerificationMethod {
    Debug = 0,
    Eudi = 1,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct AccountVerificationErrorFlags: i16 {
        const VERIFICATION_METHOD_NOT_CONFIGURED = 0x1;
        const VERIFICATION_DATA_PARSING_FAILED = 0x2;
        const VERIFICATION_DATA_VERIFICATION_FAILED = 0x4;
        const PROFILE_AGE_RANGE_VERIFICATION_FAILED = 0x8;
        const PROFILE_AGE_RANGE_VERIFICATION_MISMATCH = 0x10;
        const PROFILE_AGE_RANGE_MISMATCH = 0x20;
        const PROFILE_NAME_VERIFICATION_FAILED = 0x40;
        const PROFILE_NAME_VERIFICATION_MISMATCH = 0x80;
        const PROFILE_NAME_MISMATCH = 0x100;
        const SECURITY_CONTENT_VERIFICATION_FAILED = 0x200;
        const SECURITY_CONTENT_VERIFICATION_MISMATCH = 0x400;
        const SECURITY_CONTENT_MISMATCH = 0x800;
    }
}

impl TryFrom<i16> for AccountVerificationErrorFlags {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::from_bits(value).ok_or_else(|| "Unknown bitflag".to_string())
    }
}

/// Value for account verification error flags.
///
/// - VERIFICATION_METHOD_NOT_CONFIGURED = 0x1. Verification method is not configured.
/// - VERIFICATION_DATA_PARSING_FAILED = 0x2. Verification data parsing failed.
/// - VERIFICATION_DATA_VERIFICATION_FAILED = 0x4. Verification data verification failed.
/// - PROFILE_AGE_RANGE_VERIFICATION_FAILED = 0x8. Profile age range verification failed.
/// - PROFILE_AGE_RANGE_VERIFICATION_MISMATCH = 0x10. Value in verification data does not match
///   user set profile age range value.
/// - PROFILE_AGE_RANGE_MISMATCH = 0x20. User changed profile age range during verification
///   process.
/// - PROFILE_NAME_VERIFICATION_FAILED = 0x40. Profile name verification failed.
/// - PROFILE_NAME_VERIFICATION_MISMATCH = 0x80. Value in verification data does not match
///   user set profile name value.
/// - PROFILE_NAME_MISMATCH = 0x100. User changed profile name during verification process.
/// - SECURITY_CONTENT_VERIFICATION_FAILED = 0x200. Security content verification failed.
/// - SECURITY_CONTENT_VERIFICATION_MISMATCH = 0x400. Value in verification data does not match
///   user set security content value.
/// - SECURITY_CONTENT_MISMATCH = 0x800. User changed security content during verification
///   process.
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

impl From<AccountVerificationErrorFlagsValue> for AccountVerificationErrorFlags {
    fn from(value: AccountVerificationErrorFlagsValue) -> Self {
        Self::from_bits_retain(value.v)
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
