use base64::{Engine, prelude::BASE64_STANDARD};
use model::{PublicKeyId, UnixTime};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, IntoParams)]
pub struct AddPublicKeyParams {
    /// Ignore pending messages error. If this is true, the public key will be added
    /// even if there are pending messages.
    #[serde(default)]
    #[param(default = false)]
    pub ignore_pending_messages: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct AddPublicKeyResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    key_id: Option<PublicKeyId>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_too_many_public_keys: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_pending_messages_found: bool,
}

impl AddPublicKeyResult {
    pub fn success(key_id: PublicKeyId) -> Self {
        Self {
            key_id: Some(key_id),
            ..Default::default()
        }
    }

    pub fn error_too_many_keys() -> Self {
        Self {
            error: true,
            error_too_many_public_keys: true,
            ..Default::default()
        }
    }

    pub fn error_pending_messages_found() -> Self {
        Self {
            error: true,
            error_pending_messages_found: true,
            ..Default::default()
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
    pub max_public_key_count: i64,
}

impl GetPrivatePublicKeyInfo {
    pub fn public_key_count_limit(&self) -> i64 {
        self.max_public_key_count
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
