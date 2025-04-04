use model::PublicKeyId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct AddPublicKeyResult {
    pub key_id: Option<PublicKeyId>,
    pub error_too_many_public_keys: bool,
}

impl AddPublicKeyResult {
    pub fn success(key_id: PublicKeyId) -> Self {
        Self {
            key_id: Some(key_id),
            error_too_many_public_keys: false,
        }
    }

    pub fn error_too_many_keys() -> Self {
        Self {
            key_id: None,
            error_too_many_public_keys: true
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetLatestPublicKeyId {
    pub id: Option<PublicKeyId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetPrivatePublicKeyInfo {
    pub latest_public_key_id: Option<PublicKeyId>,
    pub max_public_key_count_from_backend_config: i64,
    pub max_public_key_count_from_account_config: i64,
}

impl GetPrivatePublicKeyInfo {
    pub fn public_key_count_limit(&self) -> i64 {
        self.max_public_key_count_from_backend_config
            .max(self.max_public_key_count_from_account_config)
    }
}
