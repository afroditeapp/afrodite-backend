use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AccountId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountVerificationQueueAdminItem {
    pub account_id: AccountId,
    pub verification_method: String,
    pub verification_data: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct GetAccountVerificationQueueNextItemResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<AccountVerificationQueueAdminItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostAccountVerificationQueueRemoveNextItem {
    pub account_id: AccountId,
}
