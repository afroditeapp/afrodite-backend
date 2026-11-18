use model_server_data::StatisticsProfileVisibility;
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, IntoParams)]
pub struct GetProfileStatisticsParams {
    /// Deprecated. Does nothing.
    /// Statistics for public and private profiles are always returned.
    ///
    /// Non default value was only for admins.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[into_params(nullable = false)]
    pub profile_visibility: Option<StatisticsProfileVisibility>,
    /// Non default value is only for admins.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[param(default = false)]
    pub generate_new_statistics: bool,
}

impl GetProfileStatisticsParams {
    pub fn contains_admin_settings(&self) -> bool {
        self.generate_new_statistics
    }
}
