use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct AppAttestation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<DebugAppAttestation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct DebugAppAttestation {
    /// [DebugAppAttestationToken] as JSON
    pub token: String,
    /// Base64 URL (with possible padding) encoded nonce.
    ///
    /// The token contains Base64 URL (with possible padding) encoded SHA-256
    /// of the nonce.
    pub nonce: String,
}

#[derive(Deserialize)]
pub struct DebugAppAttestationToken {
    pub device_integrity: bool,
    pub app_integrity: bool,
    pub nonce: String,
}
