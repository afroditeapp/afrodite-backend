use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct UserId {
    user_id: String,
}

impl UserId {
    /// TODO: validate user id?
    pub fn new(user_id: String) -> Self {
        Self { user_id }
    }

    pub fn into_string(self) -> String {
        self.user_id
    }

    pub fn as_str(&self) -> &str {
        &self.user_id
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct ApiKey {
    /// API token which server generates.
    api_key: String,
}

impl ApiKey {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn into_string(self) -> String {
        self.api_key
    }

    pub fn as_str(&self) -> &str {
        &self.api_key
    }
}
