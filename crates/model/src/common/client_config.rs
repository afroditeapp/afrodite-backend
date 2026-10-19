use base64::{Engine, prelude::BASE64_STANDARD};
use diesel::{
    Selectable,
    deserialize::Queryable,
    sql_types::{SmallInt, Text},
};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{
    AppAttestationTypeNumber, NonEmptyString, SimpleDieselEnum, diesel_non_empty_string_wrapper,
};
use utoipa::ToSchema;

use super::ClientConfigSyncVersion;
use crate::{
    AccountIdDb, ClientFeaturesConfigHash, CustomReportsConfigHash,
    DynamicClientFeaturesConfigHash, PartialProfileAttributesConfig, PortNumber,
    WebSocketClientTypeNumber,
};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClientConfig {
    /// None, if client features are not configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub client_features: Option<ClientFeaturesConfigHash>,
    /// None, if dynamic client features are not configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub dynamic_client_features: Option<DynamicClientFeaturesConfigHash>,
    /// None, if custom reports are not configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub custom_reports: Option<CustomReportsConfigHash>,
    /// None, if attributes are not configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub profile_attributes: Option<PartialProfileAttributesConfig>,
    pub sync_version: ClientConfigSyncVersion,
}

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
pub enum ClientType {
    Android = WebSocketClientTypeNumber::Android as i16,
    Ios = WebSocketClientTypeNumber::Ios as i16,
    Web = WebSocketClientTypeNumber::Web as i16,
    Bot = WebSocketClientTypeNumber::Bot as i16,
}

#[derive(Debug, Clone, Default, Serialize, ToSchema, Queryable, Selectable)]
#[diesel(table_name = crate::schema::login_session_info)]
#[diesel(check_for_backend(crate::Db))]
pub struct LoginSessionInfo {
    pub client_platform: Option<ClientType>,
    pub client_port: Option<PortNumber>,
    pub app_attestation_type_number: Option<AppAttestationTypeNumber>,
    pub app_attestation_app_integrity: Option<bool>,
    pub app_attestation_device_integrity: Option<bool>,
    pub app_attestation_failed: Option<bool>,
}

/// Apple App Attest device key stored for an account.
#[derive(Debug, Clone, Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::apple_app_attest_keys)]
#[diesel(check_for_backend(crate::Db))]
pub struct AppleAppAttestKey {
    pub account_id: AccountIdDb,
    /// SHA-256 hash of the device public key (32 bytes).
    pub key_id: Vec<u8>,
    /// Device public key extracted from the attestation certificate (65 bytes).
    pub public_key: Vec<u8>,
    /// Unix time when the key DB row was created.
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppleAppAttestKeyForDataExport {
    pub key_id_base64: String,
    pub public_key_base64: String,
    pub created_at: i64,
}

impl From<AppleAppAttestKey> for AppleAppAttestKeyForDataExport {
    fn from(key: AppleAppAttestKey) -> Self {
        Self {
            key_id_base64: BASE64_STANDARD.encode(&key.key_id),
            public_key_base64: BASE64_STANDARD.encode(&key.public_key),
            created_at: key.created_at,
        }
    }
}

#[derive(
    Debug, Clone, Deserialize, Serialize, ToSchema, diesel::FromSqlRow, diesel::AsExpression,
)]
#[diesel(sql_type = Text)]
pub struct ClientLanguage {
    // Language code like "en". Non-empty string.
    pub l: NonEmptyString,
}

impl ClientLanguage {
    pub fn new(l: NonEmptyString) -> Self {
        Self { l }
    }

    pub fn as_str(&self) -> &str {
        self.l.as_str()
    }
}

impl TryFrom<NonEmptyString> for ClientLanguage {
    type Error = String;

    fn try_from(l: NonEmptyString) -> Result<Self, Self::Error> {
        Ok(Self { l })
    }
}

diesel_non_empty_string_wrapper!(ClientLanguage);

impl AsRef<str> for ClientLanguage {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GetClientLanguage {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub l: Option<ClientLanguage>,
}
