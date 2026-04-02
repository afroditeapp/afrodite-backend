use database::current::read::GetDbReadCommandsCommon;
use model::{
    AccessTokenType, Account, AccountId, AccountIdInternal, LatestBirthdate, RefreshToken,
};
use model_server_data::SearchGroupFlags;
use server_common::data::IntoDataError;
use simple_backend_model::NonEmptyString;
use unicode_segmentation::UnicodeSegmentation;

use super::{super::DataError, DbRead};
use crate::{
    cache::CacheReadCommon, db_manager::InternalReading, define_cmd_wrapper_read, result::Result,
};

mod bot_config;
mod client_config;
mod data_export;
mod notification;
mod profile_attributes;
mod push_notification;

define_cmd_wrapper_read!(ReadCommandsCommon);

impl<'a> ReadCommandsCommon<'a> {
    pub fn bot_config(self) -> bot_config::ReadCommandsCommonBotConfig<'a> {
        bot_config::ReadCommandsCommonBotConfig::new(self.0)
    }

    pub fn client_config(self) -> client_config::ReadCommandsCommonClientConfig<'a> {
        client_config::ReadCommandsCommonClientConfig::new(self.0)
    }

    pub fn data_export(self) -> data_export::ReadCommandsCommonDataExport<'a> {
        data_export::ReadCommandsCommonDataExport::new(self.0)
    }

    pub fn profile_attributes(self) -> profile_attributes::ReadCommandsCommonProfileAttributes<'a> {
        profile_attributes::ReadCommandsCommonProfileAttributes::new(self.0)
    }

    pub fn notification(self) -> notification::ReadCommandsCommonNotification<'a> {
        notification::ReadCommandsCommonNotification::new(self.0)
    }

    pub fn push_notification(self) -> push_notification::ReadCommandsCommonPushNotification<'a> {
        push_notification::ReadCommandsCommonPushNotification::new(self.0)
    }
}

impl ReadCommandsCommon<'_> {
    pub async fn is_current_access_token_valid_for_websocket_connection(
        &self,
        id: AccountIdInternal,
        ip: std::net::IpAddr,
    ) -> Result<bool, DataError> {
        self.read_cache_common(id, |e| {
            Ok(e.is_login_session_valid_for_access_token_type(ip, AccessTokenType::Current, true))
        })
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
    ) -> Result<Option<NonEmptyString>, DataError> {
        let (name, accepted) = self
            .cache()
            .read_cache(id, |e| {
                Ok((
                    e.profile.profile_internal().profile_name.clone(),
                    e.profile
                        .profile_name_moderation_state()
                        .as_ref()
                        .map(|v| v.0.is_accepted())
                        .unwrap_or_default(),
                ))
            })
            .await
            .into_error()?;

        match name {
            None => Ok(None),
            Some(name) if accepted => Ok(Some(name)),
            Some(name) => {
                let mut letters = name.as_str().graphemes(true);
                match (letters.next(), letters.next()) {
                    (Some(first), None) => Ok(NonEmptyString::from_string(first.to_string())),
                    (Some(first), Some(_)) => {
                        Ok(NonEmptyString::from_string(format!("{first}...")))
                    }
                    _ => Ok(None),
                }
            }
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
                    gender: e.profile.state.search_group_flags,
                })
            })
            .await
            .into_error()
    }

    pub async fn is_bot(&self, id: AccountIdInternal) -> Result<bool, DataError> {
        self.cache()
            .read_cache(id, |e| Ok(e.common.other_shared_state.is_bot_account))
            .await
            .into_error()
    }

    pub async fn automatic_profile_search_happened_at_least_once(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<bool, DataError> {
        self.cache()
            .read_cache(account_id.as_id(), |e| {
                let p = &e.profile;
                Ok(p.automatic_profile_search.last_seen_unix_time().is_some())
            })
            .await
            .into_error()
    }
}

pub struct BotAndGenderInfo {
    pub is_bot: bool,
    pub gender: SearchGroupFlags,
}
