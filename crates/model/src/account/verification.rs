use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountVerificationQueueItem {
    pub verification_method: String,
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
}
