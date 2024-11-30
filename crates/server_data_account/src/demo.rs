use config::Config;
use model::{AccessibleAccount, AccountId};
use model_server_state::AccessibleAccountsInfo;
use server_data::{db_manager::RouterDatabaseReadHandle, read::GetReadCommandsCommon, result::WrappedContextExt, DataError};

use crate::read::GetReadCommandsAccount;

pub trait AccessibleAccountsInfoUtils: Sized {
    async fn into_accounts(
        self,
        read: &RouterDatabaseReadHandle,
    ) -> server_common::result::Result<Vec<AccountId>, DataError>;

    async fn contains(
        &self,
        account: AccountId,
        read: &RouterDatabaseReadHandle,
    ) -> server_common::result::Result<(), DataError>;
}

impl AccessibleAccountsInfoUtils for AccessibleAccountsInfo {
    async fn into_accounts(
        self,
        read: &RouterDatabaseReadHandle,
    ) -> server_common::result::Result<Vec<AccountId>, DataError> {
        let (accounts, demo_mode_id) = match self {
            AccessibleAccountsInfo::All => {
                let all_accounts = read.common().account_ids_vec().await?;
                return Ok(all_accounts);
            }
            AccessibleAccountsInfo::Specific {
                config_file_accounts,
                demo_mode_id,
            } => (config_file_accounts, demo_mode_id),
        };

        let related_accounts = read
            .account()
            .demo_mode_related_account_ids(demo_mode_id)
            .await?;

        Ok(accounts
            .into_iter()
            .chain(related_accounts.into_iter())
            .collect())
    }

    async fn contains(
        &self,
        account: AccountId,
        read: &RouterDatabaseReadHandle,
    ) -> server_common::result::Result<(), DataError> {
        let (accounts, demo_mode_id) = match self {
            AccessibleAccountsInfo::All => return Ok(()),
            AccessibleAccountsInfo::Specific {
                config_file_accounts,
                demo_mode_id,
            } => (config_file_accounts, demo_mode_id),
        };

        let related_accounts = read
            .account()
            .demo_mode_related_account_ids(*demo_mode_id)
            .await?;

        accounts
            .iter()
            .chain(related_accounts.iter())
            .find(|a| **a == account)
            .ok_or(DataError::NotFound.report())?;

        Ok(())
    }
}

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
                let profile = read.account_profile_utils().profile_name_and_age(internal_id).await?;
                AccessibleAccount {
                    aid: *id,
                    name: Some(profile.name),
                    age: Some(profile.age),
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
