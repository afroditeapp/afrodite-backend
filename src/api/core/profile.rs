use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::{ApiResult, ApiResultEnum};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProfileResponse {
    result: ApiResult,
    profile: Option<Profile>,
}

impl ProfileResponse {
    pub fn success(name: String) -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::Success),
            profile: Some(Profile {name} ),
        }
    }

    pub fn database_error() -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::DatabaseConnectionFailed),
            profile: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Profile {
    name: String,
}

impl Profile {
    pub fn new(name: String) -> Self {
        Self {name}
    }
}
