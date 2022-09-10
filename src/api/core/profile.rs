
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::{ApiResult, ApiResultEnum};



#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProfileResponse {
    result: ApiResult,
    name: Option<String>,
}

impl ProfileResponse {
    pub fn success(name: String) -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::Success),
            name: Some(name),
        }
    }

    pub fn database_error() -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::DatabaseConnectionFailed),
            name: None,
        }
    }
}
