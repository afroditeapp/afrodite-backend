use diesel::sql_types::SmallInt;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{SimpleDieselEnum, play_integrity::PlayIntegrityAppAttestation};

/// App attestation method used in the last login session.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    PartialEq,
    ToSchema,
    TryFromPrimitive,
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
pub enum AppAttestationTypeNumber {
    Debug = 0,
    PlayIntegrity = 1,
}

/// Result of a successful app attestation validation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppAttestationResult {
    pub attestation_type: AppAttestationTypeNumber,
    pub integrity: AppIntegrityResult,
}

/// App and device integrity results from app attestation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppIntegrityResult {
    pub app_integrity: bool,
    pub device_integrity: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct AppAttestation {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub debug: Option<DebugAppAttestation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub play_integrity: Option<PlayIntegrityAppAttestation>,
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
