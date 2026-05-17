use serde::{Deserialize, Serialize};
use simple_backend_model::NonEmptyString;
use utoipa::ToSchema;

use crate::{ContentId, ProfileAge};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EditVerificationSecurityContent {
    pub security_content: ContentId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_value: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EditVerificationProfileAgeRange {
    #[schema(value_type = i16)]
    pub current_profile_age: ProfileAge,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_value: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EditVerificationProfileName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_profile_name: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_value: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EditVerificationValues {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_content: Option<EditVerificationSecurityContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_age_range: Option<EditVerificationProfileAgeRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_name: Option<EditVerificationProfileName>,
}
