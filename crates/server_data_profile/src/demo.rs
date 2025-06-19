use model_profile::{AccessibleAccount, AccountId};
use server_data::{
    DataError,
    db_manager::{RouterDatabaseReadHandle, handle_types::Config},
};

use crate::read::GetReadProfileCommands;

pub struct DemoModeUtils;

impl DemoModeUtils {
    pub async fn with_extra_info(
        accounts: Vec<AccountId>,
        config: &Config,
        read: &RouterDatabaseReadHandle,
    ) -> server_common::result::Result<Vec<AccessibleAccount>, DataError> {
        let mut accessible_accounts = vec![];
        for id in &accounts {
            let info = if config.components().profile {
                let internal_id = read.account_id_manager().get_internal_id(*id).await?;
                let profile = read.profile().profile(internal_id).await?;
                AccessibleAccount {
                    aid: *id,
                    name: Some(profile.profile.name),
                    age: Some(profile.profile.age),
                }
            } else {
                AccessibleAccount {
                    aid: *id,
                    name: None,
                    age: None,
                }
            };
            accessible_accounts.push(info);
        }

        Ok(accessible_accounts)
    }
}
