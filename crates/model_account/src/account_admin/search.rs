use model::AccountId;
use model_server_data::EmailAddress;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Deserialize, Serialize, IntoParams)]
pub struct GetAccountIdFromEmailParams {
    pub email: EmailAddress,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetAccountIdFromEmailResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aid: Option<AccountId>,
}
