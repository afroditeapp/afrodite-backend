use model::{AccountId, ProfileAge};
use serde::{Deserialize, Serialize};
use simple_backend_model::NonEmptyString;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileAgeAndName {
    #[schema(value_type = i64)]
    pub age: ProfileAge,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<NonEmptyString>,
}

#[derive(Deserialize, ToSchema)]
pub struct SetProfileName {
    pub account: AccountId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<NonEmptyString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProfileAgeRangeVerificationAdminInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_age_range_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_age_range_verified_manual: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostProfileAgeRangeVerifiedValue {
    pub account_id: AccountId,
    /// Bot sets automatic profile age range verification value.
    /// Human admin sets manual override value.
    /// Set to None to clear the currently applicable value.
    pub value: Option<bool>,
}
