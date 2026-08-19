use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Google Play Integrity API attestation sent by the client.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct PlayIntegrityAppAttestation {
    /// Google Play Integrity API verdict token as returned by the client.
    pub token: String,
}
