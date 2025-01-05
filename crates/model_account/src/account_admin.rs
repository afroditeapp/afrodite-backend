use model::{AccountId, UnixTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

mod news;
mod search;

pub use news::*;
pub use search::*;

use crate::{AccountBanReasonCategory, AccountBanReasonDetails};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct CurrentVersions {
    pub versions: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct SetAccountBanState {
    pub account: AccountId,
    /// `Some` value bans the account and `None` value unbans the account.
    pub ban_until: Option<UnixTime>,
    pub reason_category: Option<AccountBanReasonCategory>,
    pub reason_details: Option<AccountBanReasonDetails>,
}
