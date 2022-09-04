//! HTTP API types for all servers.

pub mod core;
pub mod media;

use ::core::fmt;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use utoipa::ToSchema;

use crate::server::database::DatabaseTaskSender;

// Paths

pub const PATH_PREFIX: &str = "/api/v1/";

// Common JSON responses

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
pub enum ApiResultEnum {
    Success = 0,
    DatabaseConnectionFailed = 1,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ApiResult {
    code: u32,
    message: ApiResultEnum,
}

impl ApiResult {
    pub fn new(result: ApiResultEnum) -> Self {
        Self { code: result as u32, message: result }
    }

    pub fn success() -> Self { Self::new(ApiResultEnum::Success) }
}

// App state getters

pub trait GetDatabaseTaskSender {
    fn database(&mut self) -> &mut DatabaseTaskSender;
}
