use model_server_data::EmailAddress;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct EmailAddressStateForAdmin {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<EmailAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_change: Option<EmailAddress>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub email_change_verified: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    #[schema(default = true)]
    pub email_login_enabled: bool,
}

fn default_true() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}
