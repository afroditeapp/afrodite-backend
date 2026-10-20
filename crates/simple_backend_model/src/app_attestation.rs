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
    AppleAppAttest = 1,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct AppAttestationRequirements {
    /// Android: require Play Integrity API device integrity verdict.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub android_device_integrity: bool,
    /// Android: require Play Integrity API app integrity verdict.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub android_app_integrity: bool,
    /// iOS: require valid Apple App Attest token.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub ios_valid_token: bool,
}

/// Result of an app attestation validation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppAttestationResult {
    /// Attestation token verification succeeded.
    Success {
        attestation_type: AppAttestationTypeNumber,
        integrity: AppIntegrityResult,
    },
    /// Attestation token verification failed but login was still accepted
    /// because nothing was required.
    Failure {
        attestation_type: AppAttestationTypeNumber,
    },
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

    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub apple_app_attest: Option<AppleAppAttest>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct AppleAppAttest {
    /// Base64-encoded CBOR attestation object returned by
    /// `DCAppAttestService.attestKey`. Present when the client is attesting
    /// a new key.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub attestation: Option<String>,
    /// Base64-encoded CBOR assertion object returned by
    /// `DCAppAttestService.generateAssertion`. Present when the client is
    /// proving an already attested key.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub assertion: Option<String>,
    /// Base64-encoded key identifier returned by
    /// `DCAppAttestService.generateKey`.
    pub key_id: String,
}
