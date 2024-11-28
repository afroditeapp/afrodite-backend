use model_server_data::ProfileAttributes;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ProfileAttributesSyncVersion;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AvailableProfileAttributes {
    pub info: Option<ProfileAttributes>,
    pub sync_version: ProfileAttributesSyncVersion,
}
