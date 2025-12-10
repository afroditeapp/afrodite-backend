//! Backup transfer protocol types

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum BackupTransferClientRole {
    Target,
    Source,
}

/// Initial message from client when establishing transfer connection.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BackupTransferInitialMessage {
    pub role: BackupTransferClientRole,
    /// Access token from target client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    /// Data from target client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_data: Option<String>,
    /// SHA256 hash of target's data from source client. The hash is in hexadecimal format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_data_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BackupTransferTargetData {
    pub target_data: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BackupTransferByteCount {
    /// Use u32 to prevent integer wrapping when checking is
    /// the value inside the current transfer budget.
    pub byte_count: u32,
}
