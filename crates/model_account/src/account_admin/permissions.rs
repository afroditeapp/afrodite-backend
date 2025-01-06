


use model::{AccountId, Permissions};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetAllAdminsResult {
    pub admins: Vec<AdminInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct AdminInfo {
    pub aid: AccountId,
    pub permissions: Permissions,
}
