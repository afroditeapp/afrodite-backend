use diesel::sql_types::{SmallInt, Text};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{NonEmptyString, SimpleDieselEnum, diesel_non_empty_string_wrapper};
use utoipa::ToSchema;

use super::ClientConfigSyncVersion;
use crate::{ClientFeaturesConfigHash, CustomReportsConfigHash, PartialProfileAttributesConfig};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClientConfig {
    /// None, if client features are not configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_features: Option<ClientFeaturesConfigHash>,
    /// None, if custom reports are not configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_reports: Option<CustomReportsConfigHash>,
    /// None, if attributes are not configured.
    #[serde(skip_serializing_if = "Option::is_none")]
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
    Android = 0,
    Ios = 1,
    Web = 2,
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

diesel_non_empty_string_wrapper!(ClientLanguage);

impl AsRef<str> for ClientLanguage {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GetClientLanguage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l: Option<ClientLanguage>,
}
