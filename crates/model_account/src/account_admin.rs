use model::{AccountId, UnixTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

mod client_features;
mod client_version;
mod email;
mod news;
mod permissions;
mod search;
mod verification;

pub use client_features::*;
pub use client_version::*;
pub use email::*;
pub use news::*;
pub use permissions::*;
pub use search::*;
pub use verification::*;

use crate::{AccountBanReasonCategory, AccountBanReasonDetails};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct CurrentVersions {
    pub versions: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct SetAccountBanState {
    pub account: AccountId,
    /// `Some` value bans the account and `None` value unbans the account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_until: Option<UnixTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_category: Option<AccountBanReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_details: Option<AccountBanReasonDetails>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct AccountLockedState {
    pub locked: bool,
}
