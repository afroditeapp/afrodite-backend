use database::current::read::GetDbReadCommandsCommon;
use model::{
    AccessTokenUnixTime, Account, AccountId, AccountIdInternal, IpAddressInternal, LatestBirthdate,
    RefreshToken,
};
use model_server_data::SearchGroupFlags;
use server_common::data::IntoDataError;
use unicode_segmentation::UnicodeSegmentation;

use super::{super::DataError, DbRead};
use crate::{
    cache::CacheReadCommon, db_manager::InternalReading, define_cmd_wrapper_read, result::Result,
};

mod client_config;
mod data_export;
mod push_notification;

define_cmd_wrapper_read!(ReadCommandsCommon);

impl<'a> ReadCommandsCommon<'a> {
    pub fn client_config(self) -> client_config::ReadCommandsCommonClientConfig<'a> {
        client_config::ReadCommandsCommonClientConfig::new(self.0)
    }

    pub fn data_export(self) -> data_export::ReadCommandsCommonDataExport<'a> {
        data_export::ReadCommandsCommonDataExport::new(self.0)
    }

    pub fn push_notification(self) -> push_notification::ReadCommandsCommonPushNotification<'a> {
        push_notification::ReadCommandsCommonPushNotification::new(self.0)
    }
}

impl ReadCommandsCommon<'_> {
    pub async fn account_access_token_creation_time_from_cache(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<AccessTokenUnixTime>, DataError> {
        self.read_cache_common(id, |e| Ok(e.access_token_unix_time()))
            .await
            .into_error()
    }

    pub async fn account_access_token_ip_address_from_cache(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<IpAddressInternal>, DataError> {
        self.read_cache_common(id, |e| Ok(e.access_token_ip_address()))
            .await
            .into_error()
    }

    pub async fn account_refresh_token_from_cache(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<RefreshToken>, DataError> {
        self.read_cache_common(id, |e| Ok(e.refresh_token().cloned()))
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

    pub async fn latest_birthdate(
        &self,
        id: AccountIdInternal,
    ) -> Result<LatestBirthdate, DataError> {
        self.db_read(move |mut cmds| cmds.common().state().other_shared_state(id))
            .await
            .into_error()
            .map(|v| v.latest_birthdate())
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

    /// Only the first letter is shown from the name if it is not accepted.
    /// None is returned if name data is not available or the name is empty.
    pub async fn user_visible_profile_name_if_data_available(
        &self,
        id: impl Into<AccountId>,
    ) -> Result<Option<String>, DataError> {
        let Some((name, accepted)) = self
            .cache()
            .read_cache(id, |e| {
                Ok(e.profile.as_ref().map(|p| {
                    (
                        p.profile_internal().profile_name.clone(),
                        p.profile_name_moderation_state()
                            .as_ref()
                            .map(|v| v.0.is_accepted())
                            .unwrap_or_default(),
                    )
                }))
            })
            .await
            .into_error()?
        else {
            return Ok(None);
        };

        if name.is_empty() {
            Ok(None)
        } else if accepted {
            Ok(Some(name))
        } else if let Some(letter) = name.graphemes(true).next() {
            Ok(Some(format!("{letter}...")))
        } else {
            Ok(None)
        }
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
