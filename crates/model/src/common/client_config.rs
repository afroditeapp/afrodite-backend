use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ClientConfigSyncVersion;
use crate::{ClientFeaturesFileHash, CustomReportsFileHash, ProfileAttributeInfo};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClientConfig {
    /// Account component specific config. It is also possible
    /// that client features are not configured.
    pub client_features: Option<ClientFeaturesFileHash>,
    /// Account component specific config. It is also possible
    /// that custom reports are not configured.
    pub custom_reports: Option<CustomReportsFileHash>,
    /// Profile component specific config. It is also possible
    /// that attributes are not configured.
    pub profile_attributes: Option<ProfileAttributeInfo>,
    pub sync_version: ClientConfigSyncVersion,
}
