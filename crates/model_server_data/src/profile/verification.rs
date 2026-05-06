use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::SmallInt};
use model::VerificationStatusFilterFlags;
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i16_wrapper;
use utoipa::ToSchema;

/// Filter value for profile verification status flags.
///
/// - PROFILE_CONTENT_FACE_VERIFIED_ANY = 0x1. At least one current profile
///   picture has effective face verified value true.
/// - PROFILE_CONTENT_FACE_VERIFIED_ALL = 0x2. All current profile pictures
///   have effective face verified value true. For empty profile picture list
///   this bit must be unset.
/// - SECURITY_CONTENT_VERIFIED = 0x4. Current security content has effective
///   security verified value true.
/// - PROFILE_AGE_RANGE_VERIFIED = 0x8. Profile age range has effective
///   verification value true.
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
pub struct ProfileVerificationStatusFilter {
    #[serde(deserialize_with = "deserialize_verification_status_filter_i16")]
    value: i16,
}

fn deserialize_verification_status_filter_i16<'de, D>(deserializer: D) -> Result<i16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = i16::deserialize(deserializer)?;
    VerificationStatusFilterFlags::try_from(v)
        .map(|_| v)
        .map_err(serde::de::Error::custom)
}

impl From<ProfileVerificationStatusFilter> for VerificationStatusFilterFlags {
    fn from(value: ProfileVerificationStatusFilter) -> Self {
        // VerificationStatusFilterFlags::try_from prevents extra bits
        // so use from_bits_retain instead of from_bits_truncate.
        VerificationStatusFilterFlags::from_bits_retain(value.value)
    }
}

impl TryFrom<i16> for ProfileVerificationStatusFilter {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        VerificationStatusFilterFlags::try_from(value)?;
        Ok(Self { value })
    }
}

impl AsRef<i16> for ProfileVerificationStatusFilter {
    fn as_ref(&self) -> &i16 {
        &self.value
    }
}

diesel_i16_wrapper!(ProfileVerificationStatusFilter);
