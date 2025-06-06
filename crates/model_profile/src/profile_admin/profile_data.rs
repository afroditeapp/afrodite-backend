use model::{AccountId, ProfileAge};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileAgeAndName {
    #[schema(value_type = i64)]
    pub age: ProfileAge,
    pub name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SetProfileName {
    pub account: AccountId,
    pub name: String,
}
