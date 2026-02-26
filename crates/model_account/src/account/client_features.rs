use model::{ClientFeaturesConfig, DynamicClientFeaturesConfig};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GetClientFeaturesConfigResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ClientFeaturesConfig>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GetDynamicClientFeaturesConfigResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<DynamicClientFeaturesConfig>,
}
