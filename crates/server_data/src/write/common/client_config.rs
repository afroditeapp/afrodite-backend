use database::current::write::GetDbWriteCommandsCommon;
use model::{AccountIdInternal, ClientLanguage, ClientType, DynamicClientFeaturesConfig};

use crate::{
    DataError, db_manager::InternalWriting, db_transaction, define_cmd_wrapper_write,
    dynamic_client_features::DynamicClientFeatures, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsCommonClientConfig);

impl WriteCommandsCommonClientConfig<'_> {
    pub async fn upsert_dynamic_client_features_config(
        &self,
        config: &DynamicClientFeaturesConfig,
    ) -> Result<(), DataError> {
        let config = config.clone();
        let (hash, config) = db_transaction!(self, move |mut cmds| {
            let hash = cmds
                .common()
                .client_config()
                .upsert_dynamic_client_features_config(&config)?;
            cmds.common()
                .client_config()
                .increment_client_config_sync_version_for_every_account()?;
            Ok((hash, config))
        })?;

        self.dynamic_client_features()
            .set_dynamic_client_features(Some(DynamicClientFeatures { hash, config }))
            .await;

        Ok(())
    }

    /// Only server WebSocket code should call this method.
    pub async fn reset_client_config_sync_version(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .client_config()
                .reset_client_config_sync_version(id)
        })
    }

    pub async fn increment_client_config_sync_version_for_every_account(
        &self,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .client_config()
                .increment_client_config_sync_version_for_every_account()
        })
    }

    pub async fn client_login_session_platform(
        &self,
        id: AccountIdInternal,
        value: ClientType,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .client_config()
                .update_client_login_session_platform(id, value)
        })
    }

    pub async fn client_language(
        &self,
        id: AccountIdInternal,
        value: ClientLanguage,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .client_config()
                .update_client_language(id, value)
        })
    }
}
