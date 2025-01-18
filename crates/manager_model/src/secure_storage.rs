use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct SecureStorageEncryptionKey {
    /// Base64 key
    pub key: String,
}
