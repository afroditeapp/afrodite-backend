use model::{
    AccountId, AccountIdDb, AccountIdInternal, AccountState, AccountStateRelatedSharedState,
    AppAttestationTypeNumber, ClientType, LoginMethod, ProfileAge, ProfileVisibility,
};
use model_server_data::ProfileInternal;
use serde::{Deserialize, Serialize};
use simple_backend_model::NonEmptyString;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountIdDbValue {
    pub account_db_id: AccountIdDb,
}

/// Account list filters.
///
/// Filter values which the caller does not have permission to use are
/// silently ignored.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct AccountFilters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub account_state: Option<AccountState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub account_online: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub profile_visibility: Option<ProfileVisibility>,
    /// Includes accounts whose latest login session IP country is not
    /// the provided country code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub login_session_ip_country_is_not: Option<String>,
    /// Includes accounts whose latest login session does not have valid
    /// basic app attestation. Valid basic app attestation means valid
    /// iOS app attestation token or valid Android app and device
    /// integrity values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub login_session_basic_app_attestation_not_valid: Option<bool>,
}

impl AccountFilters {
    pub fn any_login_session_filter(&self) -> bool {
        self.login_session_ip_country_is_not.is_some()
            || self.login_session_basic_app_attestation_not_valid.is_some()
    }
}

/// Account list extra info settings.
///
/// Requested data which the caller does not have permission to view is
/// omitted from the response.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, ToSchema)]
pub struct AccountExtraInfoSettings {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub account_state: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub account_online: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_visibility: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub email_address: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub login_session_client_type: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub login_session_login_method: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub login_session_app_attestation_info: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub login_session_ip_country: bool,
}

impl AccountExtraInfoSettings {
    pub fn any_login_session_info(&self) -> bool {
        self.login_session_client_type
            || self.login_session_login_method
            || self.login_session_app_attestation_info
            || self.login_session_ip_country
    }
}

/// Params for admin profile iterator page.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProfileIteratorSettings {
    pub start_position: AccountIdDb,
    pub page: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub filters: Option<AccountFilters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub extra_info: Option<AccountExtraInfoSettings>,
}

impl ProfileIteratorSettings {
    pub fn filters(&self) -> AccountFilters {
        self.filters.clone().unwrap_or_default()
    }

    pub fn extra_info(&self) -> AccountExtraInfoSettings {
        self.extra_info.unwrap_or_default()
    }
}

/// Database data for one profile iterator page value. Internal type.
#[derive(Debug, Clone)]
pub struct ProfileIteratorPageData {
    pub id: AccountIdInternal,
    pub profile: ProfileInternal,
    pub state: AccountStateRelatedSharedState,
}

/// Login session info for account extra info.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminAccountLoginSessionInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub client_type: Option<ClientType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub login_method: Option<LoginMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub app_attestation_type: Option<AppAttestationTypeNumber>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub app_attestation_app_integrity: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub app_attestation_device_integrity: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub app_attestation_failed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProfileIteratorPageValue {
    pub account_id: AccountId,
    #[schema(value_type = i64)]
    pub age: ProfileAge,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub name: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub account_state: Option<AccountState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub account_online: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub profile_visibility: Option<ProfileVisibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub login_session: Option<AdminAccountLoginSessionInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub login_session_ip_country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProfileIteratorPage {
    pub values: Vec<ProfileIteratorPageValue>,
    /// Page number of the next page. None if the end of the iteration
    /// has been reached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub next_page: Option<i64>,
}
