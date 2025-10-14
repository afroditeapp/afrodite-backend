use database::current::write::GetDbWriteCommandsCommon;
use model::{Account, AccountIdInternal, ReportTypeNumberInternal, UnixTime};
use server_common::data::cache::CacheError;
use simple_backend_utils::time::DurationValue;

use super::{DbTransaction, GetWriteCommandsCommon};
use crate::{
    DataError, IntoDataError, cache::CacheWriteCommon, db_manager::InternalWriting, db_transaction,
    define_cmd_wrapper_write, file::FileWrite, result::Result,
};

mod client_config;
mod data_export;
mod push_notification;

define_cmd_wrapper_write!(WriteCommandsCommon);

impl WriteCommandsCommon<'_> {
    pub fn push_notification(&mut self) -> push_notification::WriteCommandsCommonPushNotification {
        push_notification::WriteCommandsCommonPushNotification::new(self.handle())
    }

    pub fn client_config(&mut self) -> client_config::WriteCommandsCommonClientConfig {
        client_config::WriteCommandsCommonClientConfig::new(self.handle())
    }

    pub fn data_export(&mut self) -> data_export::WriteCommandsCommonDataExport {
        data_export::WriteCommandsCommonDataExport::new(self.handle())
    }
}

impl WriteCommandsCommon<'_> {
    pub async fn save_authentication_tokens_from_cache_to_db_if_needed(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        let Some(login_session) = self
            .cache()
            .write_cache_common(id, |e| Ok(e.get_tokens_if_save_needed()))
            .await?
        else {
            return Ok(());
        };

        db_transaction!(self, move |mut cmds| {
            cmds.common().token().login_session(id, login_session)?;
            Ok(())
        })?;

        Ok(())
    }

    pub async fn logout(&self, id: AccountIdInternal) -> Result<(), DataError> {
        self.cache().logout(id.into()).await.into_data_error(id)?;

        self.handle()
            .common()
            .push_notification()
            .remove_push_notification_device_token_and_encryption_key(id)
            .await?;

        Ok(())
    }

    pub async fn remove_tmp_files(&self, id: AccountIdInternal) -> Result<(), DataError> {
        self.files()
            .tmp_dir(id.into())
            .overwrite_and_remove_contents_if_exists()
            .await
            .into_data_error(id)
    }

    pub async fn set_is_bot_account(
        &self,
        id: AccountIdInternal,
        value: bool,
    ) -> Result<(), DataError> {
        self.write_cache_common(id, |cache| {
            cache.other_shared_state.is_bot_account = value;
            Ok(())
        })
        .await?;

        db_transaction!(self, move |mut cmds| {
            cmds.common().state().set_is_bot_account(id, value)
        })
    }

    pub async fn internal_handle_new_account_data_after_db_modification(
        &self,
        id: AccountIdInternal,
        current_account: &Account,
        new_account: &Account,
    ) -> Result<(), DataError> {
        let new_account_clone = new_account.clone();
        self.write_cache_common(id, |cache| {
            cache.permissions = new_account_clone.permissions();
            cache.account_state_related_shared_state = new_account_clone.into();
            Ok(())
        })
        .await?;

        // Other related state updating

        if current_account.profile_visibility().is_currently_public()
            != new_account.profile_visibility().is_currently_public()
        {
            self.profile_update_location_index_visibility(
                id,
                new_account.profile_visibility().is_currently_public(),
            )
            .await?;
        }

        Ok(())
    }

    pub async fn update_initial_setup_completed_unix_time(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        let time = db_transaction!(self, move |mut cmds| {
            cmds.common()
                .state()
                .update_initial_setup_completed_unix_time(id)
        })?;

        self.write_cache_common(id, |e| {
            e.other_shared_state.initial_setup_completed_unix_time = time;
            Ok(())
        })
        .await?;

        Ok(())
    }

    pub async fn delete_processed_reports_if_needed(
        &self,
        report_type: ReportTypeNumberInternal,
        deletion_wait_time: DurationValue,
    ) -> Result<(), DataError> {
        let automatic_deletion_allowed =
            UnixTime::current_time().add_seconds(deletion_wait_time.seconds);

        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .report()
                .delete_old_reports(report_type, automatic_deletion_allowed)?;
            Ok(())
        })?;

        Ok(())
    }
}

pub trait UpdateLocationIndexVisibility {
    async fn profile_update_location_index_visibility(
        &self,
        id: AccountIdInternal,
        visibility: bool,
    ) -> Result<(), DataError>;
}

impl<I: InternalWriting> UpdateLocationIndexVisibility for I {
    async fn profile_update_location_index_visibility(
        &self,
        id: AccountIdInternal,
        visibility: bool,
    ) -> Result<(), DataError> {
        let (location, profile_data) = self
            .cache()
            .read_cache(id.as_id(), |e| {
                let index_data = e.location_index_profile_data();
                let p = &e.profile;

                Ok::<
                    (
                        model_server_data::LocationIndexKey,
                        model_server_data::LocationIndexProfileData,
                    ),
                    error_stack::Report<CacheError>,
                >((p.location.current_position.profile_location(), index_data))
            })
            .await
            .into_data_error(id)?;

        if visibility {
            self.location_index_write_handle()
                .update_profile_data(id.as_id(), profile_data, location)
                .await?;
        } else {
            self.location_index_write_handle()
                .remove_profile_data(id.as_id(), location)
                .await?;
        }

        Ok(())
    }
}
