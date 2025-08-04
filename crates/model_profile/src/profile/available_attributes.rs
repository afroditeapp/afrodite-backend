use model::AttributeId;
use model_server_data::ProfileAttributesConfigQueryItem;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileAttributesConfigQuery {
    pub values: Vec<AttributeId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileAttributesConfigQueryResult {
    pub values: Vec<ProfileAttributesConfigQueryItem>,
}
