use model::InfoBannersConfig;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct SaveInfoBanners {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<InfoBannersConfig>,
    pub new: InfoBannersConfig,
}
