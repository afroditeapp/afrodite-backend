use chrono::NaiveDate;
use database::current::read::GetDbReadCommandsCommon;
use model::{
    AccessToken, Account, AccountId, AccountIdInternal, PendingNotificationFlags, RefreshToken,
    ReportAccountInfo,
};
use model_server_data::SearchGroupFlags;
use server_common::data::IntoDataError;

use super::{super::DataError, DbRead};
use crate::{
    cache::CacheReadCommon, db_manager::InternalReading, define_cmd_wrapper_read,
    id::ToAccountIdInternal, result::Result,
};

mod client_config;

define_cmd_wrapper_read!(ReadCommandsCommon);

impl<'a> ReadCommandsCommon<'a> {
    pub fn client_config(self) -> client_config::ReadCommandsCommonClientConfig<'a> {
        client_config::ReadCommandsCommonClientConfig::new(self.0)
    }
}

impl ReadCommandsCommon<'_> {
    pub async fn account_access_token(
        &self,
        id: AccountId,
    ) -> Result<Option<AccessToken>, DataError> {
        let id = self.to_account_id_internal(id).await.into_data_error(id)?;
        self.db_read(move |mut cmds| cmds.common().token().access_token(id))
            .await
            .into_error()
    }

    pub async fn account_refresh_token(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<RefreshToken>, DataError> {
        self.db_read(move |mut cmds| cmds.common().token().refresh_token(id))
            .await
            .into_error()
    }

    /// Account is available on all servers as account server will sync it to
    /// others if server is running in microservice mode.
    pub async fn account(&self, id: AccountIdInternal) -> Result<Account, DataError> {
        let account = self
            .read_cache_common(id, |cache| {
                Ok(Account::new_from_internal_types(
                    cache.permissions.clone(),
                    cache.account_state_related_shared_state.clone(),
                ))
            })
            .await?;
        Ok(account)
    }

    pub async fn cached_pending_notification_flags(
        &self,
        id: AccountIdInternal,
    ) -> Result<PendingNotificationFlags, DataError> {
        let flags = self
            .read_cache_common(id, |cache| Ok(cache.pending_notification_flags))
            .await?;
        Ok(flags)
    }

    pub async fn latest_birthdate(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<NaiveDate>, DataError> {
        self.db_read(move |mut cmds| cmds.common().state().other_shared_state(id))
            .await
            .into_error()
            .map(|v| v.birthdate)
    }

    pub async fn account_ids_vec(&self) -> Result<Vec<AccountId>, DataError> {
        self.db_read(move |mut cmds| cmds.common().account_ids())
            .await
            .into_error()
    }

    pub async fn account_ids_internal_vec(&self) -> Result<Vec<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| cmds.common().account_ids_internal())
            .await
            .into_error()
    }

    pub async fn account_ids_for_logged_in_clients(&self) -> Vec<AccountIdInternal> {
        self.cache().logged_in_clients().await
    }

    pub async fn backup_current_database(&self, file_name: String) -> Result<(), DataError> {
        self.db_read_raw_no_transaction(move |mut cmds| {
            cmds.common().backup_current_database(file_name)
        })
        .await
        .into_error()
    }

    pub async fn push_notification_already_sent(
        &self,
        id: AccountIdInternal,
    ) -> Result<bool, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common()
                .push_notification()
                .push_notification_already_sent(id)
        })
        .await
        .into_error()
    }

    pub async fn get_profile_age_and_name_if_profile_component_is_enabled(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<ReportAccountInfo>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common()
                .report()
                .get_report_account_info(*id.as_db_id())
        })
        .await
        .into_error()
    }

    pub async fn bot_and_gender_info(
        &self,
        id: AccountIdInternal,
    ) -> Result<BotAndGenderInfo, DataError> {
        self.cache()
            .read_cache(id, |e| {
                Ok(BotAndGenderInfo {
                    is_bot: e.common.other_shared_state.is_bot_account,
                    gender: e
                        .profile
                        .as_ref()
                        .map(|p| p.state.search_group_flags)
                        .unwrap_or(SearchGroupFlags::empty()),
                })
            })
            .await
            .into_error()
    }
}

pub struct BotAndGenderInfo {
    pub is_bot: bool,
    pub gender: SearchGroupFlags,
}
