use model_server_data::MatchesIteratorSessionId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ChatProfileLink;
use crate::AccountId;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct AllMatchesPage {
    pub profiles: Vec<AccountId>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResetMatchesIteratorResult {
    pub s: MatchesIteratorSessionId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct MatchesPage {
    pub p: Vec<ChatProfileLink>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_invalid_iterator_session_id: bool,
}
