use model::{AccountId, ReportProcessingState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct ProfileReport {
    pub processing_state: ReportProcessingState,
    pub profile_text: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateProfileReport {
    pub target: AccountId,
    pub profile_text: Option<String>,
}
