use model::ClientFeaturesConfig;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GetClientFeaturesConfigResult {
    pub config: Option<ClientFeaturesConfig>,
}
