use model_server_data::{AttributeId, ProfileAttributeInfo, ProfileAttributeQueryItem};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ProfileAttributesSyncVersion;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AvailableProfileAttributes {
    pub info: Option<ProfileAttributeInfo>,
    pub sync_version: ProfileAttributesSyncVersion,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileAttributeQuery {
    pub values: Vec<AttributeId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileAttributeQueryResult {
    pub values: Vec<ProfileAttributeQueryItem>,
}
