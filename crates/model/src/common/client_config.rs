use diesel::sql_types::Text;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_string_wrapper};
use utoipa::ToSchema;

use super::ClientConfigSyncVersion;
use crate::{
    ClientFeaturesConfigHash, CustomReportsConfigHash, PartialProfileAttributesConfig,
    schema_sqlite_types::Integer,
};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClientConfig {
    /// Account component specific config. It is also possible
    /// that client features are not configured.
    pub client_features: Option<ClientFeaturesConfigHash>,
    /// Account component specific config. It is also possible
    /// that custom reports are not configured.
    pub custom_reports: Option<CustomReportsConfigHash>,
    /// Profile component specific config. It is also possible
    /// that attributes are not configured.
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
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Integer)]
#[repr(i64)]
pub enum ClientType {
    Android = 0,
    Ios = 1,
    Web = 2,
}

diesel_i64_try_from!(ClientType);

#[derive(
    Debug,
    Default,
    Clone,
    Deserialize,
    Serialize,
    ToSchema,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Text)]
pub struct ClientLanguage {
    // Language code like "en". Might be empty.
    pub l: String,
}

impl ClientLanguage {
    pub fn new(l: String) -> Self {
        Self { l }
    }

    pub fn as_str(&self) -> &str {
        &self.l
    }
}

diesel_string_wrapper!(ClientLanguage);
