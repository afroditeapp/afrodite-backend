use model::ClientFeaturesConfig;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GetClientFeaturesConfigResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ClientFeaturesConfig>,
}
