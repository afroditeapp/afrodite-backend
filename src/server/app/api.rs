
use serde::{Deserialize, Serialize};

// Paths

pub const PATH_REGISTER: &str = "/api/v1/register";

// Common

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum ApiResultEnum {
    Success = 0,
    DatabaseConnectionFailed = 1,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResult {
    code: u32,
    message: ApiResultEnum,
}

impl ApiResult {
    pub fn new(result: ApiResultEnum) -> Self {
        Self { code: result as u32, message: result }
    }
}

// HTTP post

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateProfile {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateProfileResponse {
    result: ApiResult,
    profile_id: Option<String>,
}

impl CreateProfileResponse {
    pub fn success(profile_id: String) -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::Success),
            profile_id: Some(profile_id)
        }
    }

    pub fn error() -> Self {
        Self {
            result: ApiResult::new(ApiResultEnum::DatabaseConnectionFailed),
            profile_id: None,
        }
    }
}
