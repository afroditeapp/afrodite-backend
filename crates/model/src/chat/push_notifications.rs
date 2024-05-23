
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_string_wrapper;
use utoipa::{IntoParams, ToSchema};

use crate::{schema::shared_state, schema_sqlite_types::Integer, AccessToken, AccountIdDb, AccountIdInternal, AccountSyncVersion, RefreshToken, SharedStateRaw};

/// Firebase Cloud Messaging device token.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
#[serde(transparent)]
pub struct FcmDeviceToken(pub String);

impl FcmDeviceToken {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

diesel_string_wrapper!(FcmDeviceToken);
