use model_profile::{AccessibleAccount, AccountId};
use server_data::{DataError, db_manager::RouterDatabaseReadHandle};

use crate::read::GetReadProfileCommands;

pub struct DemoAccountUtils;

impl DemoAccountUtils {
    pub async fn with_extra_info(
        accounts: Vec<AccountId>,
        read: &RouterDatabaseReadHandle,
    ) -> server_common::result::Result<Vec<AccessibleAccount>, DataError> {
        let mut accessible_accounts = vec![];
        for id in &accounts {
            let internal_id = read.account_id_manager().get_internal_id(*id).await?;
            let profile = read.profile().profile(internal_id).await?;
            let info = AccessibleAccount {
                aid: *id,
                name: Some(profile.profile.name),
                age: Some(profile.profile.age),
            };
            accessible_accounts.push(info);
        }

        Ok(accessible_accounts)
    }
}
