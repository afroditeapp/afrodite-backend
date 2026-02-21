use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ProfileAttributesSchemaExport;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateProfileAttributesSchema {
    pub current_state: ProfileAttributesSchemaExport,
    pub new_state: ProfileAttributesSchemaExport,
}
