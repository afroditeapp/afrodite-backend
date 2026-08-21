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
    PlayIntegrity = 0,
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
    pub play_integrity: Option<PlayIntegrityAppAttestation>,
}
