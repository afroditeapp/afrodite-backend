use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::{ApiResult, ApiResultEnum};

pub type UserId = String;
pub type UserApiToken = String;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterBody {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterResponse {
    result: ApiResult,
    user_id: Option<UserId>,
}

impl RegisterResponse {
    pub fn success(user_id: UserId) -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::Success),
            user_id: Some(user_id),
        }
    }

    pub fn database_error() -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::DatabaseConnectionFailed),
            user_id: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginBody {
    pub user_id: UserId,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginResponse {
    result: ApiResult,
    /// API key which server generates.
    api_key: Option<UserApiToken>,
}

impl LoginResponse {
    pub fn success(api_key: UserApiToken) -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::Success),
            api_key: Some(api_key),
        }
    }

    pub fn database_error() -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::DatabaseConnectionFailed),
            api_key: None,
        }
    }
}
