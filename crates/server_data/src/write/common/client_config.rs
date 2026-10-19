use database::current::write::GetDbWriteCommandsCommon;
use model::{
    AccountIdInternal, AppAttestationResult, ClientLanguage, ClientType,
    DynamicClientFeaturesConfig, DynamicServerConfig, IpAddressInternal, PortNumber,
};

use crate::{
    DataError, db_manager::InternalWriting, db_transaction, define_cmd_wrapper_write,
    dynamic_client_features::DynamicClientFeatures, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsCommonClientConfig);

impl WriteCommandsCommonClientConfig<'_> {
    pub async fn upsert_dynamic_client_features_config(
        &self,
        config: DynamicClientFeaturesConfig,
    ) -> Result<(), DataError> {
        let manager = self.dynamic_client_features().clone();
        db_transaction!(self, move |mut cmds| {
            let hash = cmds
                .common()
                .client_config()
                .upsert_dynamic_client_features_config(&config)?;
            cmds.common()
                .client_config()
                .increment_client_config_sync_version_for_every_account()?;
            manager
                .set_dynamic_client_features_blocking(Some(DynamicClientFeatures { hash, config }));
            Ok(())
        })?;
        Ok(())
    }

    pub async fn upsert_dynamic_server_config(
        &self,
        mut config: DynamicServerConfig,
    ) -> Result<(), DataError> {
        for domain in &mut config.email_registration_domain_lists.allowlist {
            *domain = domain.trim().to_lowercase();
        }
        for domain in &mut config.email_registration_domain_lists.blocklist {
            *domain = domain.trim().to_lowercase();
        }

        let manager = self.dynamic_server_config().clone();
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .client_config()
                .upsert_dynamic_server_config(&config)?;
            manager.set_dynamic_server_config_blocking(Some(config));
            Ok(())
        })?;
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

    pub async fn login_session_info(
        &self,
        id: AccountIdInternal,
        platform: ClientType,
        port: Option<PortNumber>,
        ip: IpAddressInternal,
        attestation: Option<AppAttestationResult>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common().client_config().update_login_session_info(
                id,
                platform,
                port,
                ip,
                attestation,
            )
        })
    }

    /// Insert a new Apple App Attest key for an account and limit the total
    /// number of keys to `max_keys` by deleting the oldest ones.
    pub async fn insert_apple_app_attest_key(
        &self,
        id: AccountIdInternal,
        key_id: &[u8],
        public_key: &[u8],
        created_at: i64,
        max_keys: i64,
    ) -> Result<(), DataError> {
        let key_id = key_id.to_vec();
        let public_key = public_key.to_vec();
        db_transaction!(self, move |mut cmds| {
            cmds.common().client_config().insert_apple_app_attest_key(
                id,
                &key_id,
                &public_key,
                created_at,
            )?;
            cmds.common()
                .client_config()
                .limit_apple_app_attest_keys(id, max_keys)
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
