
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClientFeaturesFileHash {
    h: String,
}

impl ClientFeaturesFileHash {
    pub fn new(h: String) -> Self {
        Self { h }
    }

    pub fn hash(&self) -> &str {
        &self.h
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClientFeaturesConfig {
    #[serde(default)]
    pub features: FeaturesConfig,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeaturesConfig {
    /// Enable news UI
    pub news: bool,
}
