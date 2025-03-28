use diesel::prelude::*;
use model_server_data::ProfileNameModerationState;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{AccountId, AccountIdDb};

#[derive(Debug, Clone, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::profile_name_allowlist)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct ProfileNameAllowlist {
    pub name_creator_account_id: AccountIdDb,
    pub name_moderator_account_id: Option<AccountIdDb>,
    pub profile_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileNamePendingModerationList {
    pub values: Vec<ProfileNamePendingModeration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct ProfileNamePendingModeration {
    pub id: AccountId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct PostModerateProfileName {
    pub id: AccountId,
    pub name: String,
    pub accept: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileNameState {
    /// If empty, the profile name is not set.
    pub name: String,
    pub state: ProfileNameModerationState,
}
