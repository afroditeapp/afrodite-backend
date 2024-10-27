
use diesel::prelude::*;

use crate::AccountIdDb;

#[derive(Debug, Clone, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::profile_name_allowlist)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct ProfileNameAllowlist {
    pub name_creator_account_id: AccountIdDb,
    pub name_moderator_account_id: Option<AccountIdDb>,
    pub profile_name: String,
}
