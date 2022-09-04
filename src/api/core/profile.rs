use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::{ApiResult, ApiResultEnum, self};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterBody {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterResponse {
    result: ApiResult,
    profile_id: Option<String>,
}

impl RegisterResponse {
    pub fn success(profile_id: String) -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::Success),
            profile_id: Some(profile_id)
        }
    }

    pub fn database_error() -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::DatabaseConnectionFailed),
            profile_id: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginBody {
    pub profile_id: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginResponse {
    result: ApiResult,
    /// API key which server generates.
    api_key: Option<String>,
}

impl LoginResponse {
    pub fn success(api_key: String) -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::Success),
            api_key: Some(api_key)
        }
    }

    pub fn database_error() -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::DatabaseConnectionFailed),
            api_key: None,
        }
    }
}
