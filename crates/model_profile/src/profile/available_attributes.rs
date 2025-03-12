use model::AttributeId;
use model_server_data::ProfileAttributeQueryItem;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileAttributeQuery {
    pub values: Vec<AttributeId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileAttributeQueryResult {
    pub values: Vec<ProfileAttributeQueryItem>,
}
