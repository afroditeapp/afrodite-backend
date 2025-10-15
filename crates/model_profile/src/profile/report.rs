use model::AccountId;
use serde::{Deserialize, Serialize};
use simple_backend_model::NonEmptyString;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateProfileNameReport {
    pub target: AccountId,
    pub profile_name: NonEmptyString,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateProfileTextReport {
    pub target: AccountId,
    pub profile_text: String,
}
