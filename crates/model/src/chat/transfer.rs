//! Data transfer protocol types

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Client role in data transfer
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum ClientRole {
    Target,
    Source,
}

/// Initial message from client when establishing transfer connection.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DataTransferInitialMessage {
    pub role: ClientRole,
    /// Access token from target client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    /// Public key from target client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// Account ID from source client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// Password from target and source clients.
    ///
    /// Target sets the required password and Source must know it.
    /// The password exists to avoid constant polling to find
    /// new waiting Target clients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DataTransferPublicKey {
    pub public_key: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DataTransferByteCount {
    /// Use u32 to prevent integer wrapping when checking is
    /// the value inside the current transfer budget.
    pub byte_count: u32,
}
