use model::ClientFeaturesConfig;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetClientFeaturesConfigResult {
    pub config: Option<ClientFeaturesConfig>,
}
