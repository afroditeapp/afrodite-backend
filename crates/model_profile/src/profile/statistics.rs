use model_server_data::StatisticsProfileVisibility;
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, IntoParams)]
pub struct GetProfileStatisticsParams {
    /// Control which profiles are included in
    /// [GetProfileStatisticsResult::age_counts]
    /// by profile visibility.
    ///
    /// Non default value is only for admins.
    pub profile_visibility: Option<StatisticsProfileVisibility>,
    /// Non default value is only for admins.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[param(default = false)]
    pub generate_new_statistics: bool,
}

impl GetProfileStatisticsParams {
    pub fn contains_admin_settings(&self) -> bool {
        !self
            .profile_visibility
            .unwrap_or_default()
            .is_default_statistics()
            || self.generate_new_statistics
    }
}
