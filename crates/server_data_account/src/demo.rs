use model::{AccessibleAccount, AccountId};
use model_server_state::AccessibleAccountsInfo;
use server_data::{DataError, db_manager::RouterDatabaseReadHandle, result::WrappedContextExt};

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
        let AccessibleAccountsInfo {
            config_file_accounts,
            demo_account_id,
        } = self;

        let database_accounts = read
            .account()
            .demo_account_owned_account_ids(demo_account_id)
            .await?;

        Ok(config_file_accounts
            .into_iter()
            .chain(database_accounts)
            .collect())
    }

    async fn contains(
        &self,
        account: AccountId,
        read: &RouterDatabaseReadHandle,
    ) -> server_common::result::Result<(), DataError> {
        let AccessibleAccountsInfo {
            config_file_accounts,
            demo_account_id,
        } = self;

        let related_accounts = read
            .account()
            .demo_account_owned_account_ids(*demo_account_id)
            .await?;

        config_file_accounts
            .iter()
            .chain(related_accounts.iter())
            .find(|a| **a == account)
            .ok_or(DataError::NotFound.report())?;

        Ok(())
    }
}

pub struct DemoAccountUtils;

impl DemoAccountUtils {
    pub async fn with_extra_info(
        accounts: Vec<AccountId>,
        read: &RouterDatabaseReadHandle,
    ) -> server_common::result::Result<Vec<AccessibleAccount>, DataError> {
        let mut accessible_accounts = vec![];
        for id in &accounts {
            let internal_id = read.account_id_manager().get_internal_id(*id).await?;
            let profile = read
                .account_profile_utils()
                .profile_name_and_age(internal_id)
                .await?;
            let info = AccessibleAccount {
                aid: *id,
                name: profile.name,
                age: Some(profile.age),
            };
            accessible_accounts.push(info);
        }

        Ok(accessible_accounts)
    }
}
