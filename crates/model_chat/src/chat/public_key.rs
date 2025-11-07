use base64::{Engine, prelude::BASE64_STANDARD};
use model::{PublicKeyId, UnixTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct AddPublicKeyResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    key_id: Option<PublicKeyId>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_too_many_public_keys: bool,
}

impl AddPublicKeyResult {
    pub fn success(key_id: PublicKeyId) -> Self {
        Self {
            key_id: Some(key_id),
            error: false,
            error_too_many_public_keys: false,
        }
    }

    pub fn error_too_many_keys() -> Self {
        Self {
            key_id: None,
            error: true,
            error_too_many_public_keys: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetLatestPublicKeyId {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<PublicKeyId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetPrivatePublicKeyInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Serialize)]
pub struct DataExportPublicKey {
    public_key_id: PublicKeyId,
    binary_pgp_public_key_base64: String,
    key_added_time: UnixTime,
}

impl DataExportPublicKey {
    pub fn new(public_key_id: PublicKeyId, key_data: Vec<u8>, key_added_time: UnixTime) -> Self {
        Self {
            public_key_id,
            binary_pgp_public_key_base64: BASE64_STANDARD.encode(key_data),
            key_added_time,
        }
    }
}
