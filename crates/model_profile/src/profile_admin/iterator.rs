use model::{AccountId, AccountIdDb, ProfileAge};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountIdDbValue {
    pub account_db_id: AccountIdDb,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams)]
pub struct ProfileIteratorSettings {
    pub start_position: AccountIdDb,
    pub page: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProfileIteratorPageValue {
    pub account_id: AccountId,
    #[schema(value_type = i64)]
    pub age: ProfileAge,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProfileIteratorPage {
    pub values: Vec<ProfileIteratorPageValue>,
}
