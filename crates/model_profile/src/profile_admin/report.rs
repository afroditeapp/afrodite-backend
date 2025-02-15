use model::{AccountId, ReportDetailedInfo};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Profile name

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileNameReportDetailed {
    pub info: ReportDetailedInfo,
    pub profile_name: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct GetProfileNameReportList {
    pub values: Vec<ProfileNameReportDetailed>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProcessProfileNameReport {
    pub creator: AccountId,
    pub target: AccountId,
    pub profile_name: String,
}

// Profile text

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileTextReportDetailed {
    pub info: ReportDetailedInfo,
    pub profile_text: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct GetProfileTextReportList {
    pub values: Vec<ProfileTextReportDetailed>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProcessProfileTextReport {
    pub creator: AccountId,
    pub target: AccountId,
    pub profile_text: String,
}
