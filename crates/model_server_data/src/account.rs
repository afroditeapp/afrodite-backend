use model::{AccessToken, RefreshToken};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

mod news;
pub use news::*;

/// AccessToken and RefreshToken
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct AuthPair {
    pub refresh: RefreshToken,
    pub access: AccessToken,
}

impl AuthPair {
    pub fn new(refresh: RefreshToken, access: AccessToken) -> Self {
        Self { refresh, access }
    }
}
