use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::ProfileAttributeInfo;

use super::ClientConfigSyncVersion;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClientConfig {
    /// Profile component specific config. It is also possible
    /// that attributes are not configured.
    pub profile_attributes: Option<ProfileAttributeInfo>,
    pub sync_version: ClientConfigSyncVersion,
}
