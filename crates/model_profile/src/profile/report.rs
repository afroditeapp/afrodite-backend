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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateProfileReportResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_outdated_profile_text: bool,
}

impl UpdateProfileReportResult {
    pub fn success() -> Self {
        Self {
            error_outdated_profile_text: false,
        }
    }

    pub fn outdated_profile_text() -> Self {
        Self {
            error_outdated_profile_text: true
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileReportDetailed {
    pub creator: AccountId,
    pub target: AccountId,
    pub processing_state: ReportProcessingState,
    pub profile_text: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct GetProfileReportList {
    pub values: Vec<ProfileReportDetailed>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProcessProfileReport {
    pub creator: AccountId,
    pub target: AccountId,
    pub profile_text: Option<String>,
}
