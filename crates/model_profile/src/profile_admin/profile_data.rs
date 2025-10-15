use model::{AccountId, ProfileAge};
use serde::{Deserialize, Serialize};
use simple_backend_model::NonEmptyString;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileAgeAndName {
    #[schema(value_type = i64)]
    pub age: ProfileAge,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<NonEmptyString>,
}

#[derive(Deserialize, ToSchema)]
pub struct SetProfileName {
    pub account: AccountId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<NonEmptyString>,
}
