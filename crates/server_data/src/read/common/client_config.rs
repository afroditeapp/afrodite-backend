use database::current::read::GetDbReadCommandsCommon;
use model::{
    AccountIdInternal, AppleAppAttestKey, ClientConfigSyncVersion, ClientLanguage, ClientType,
    LoginSessionInfo,
};

use crate::{DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result};

define_cmd_wrapper_read!(ReadCommandsCommonClientConfig);

impl ReadCommandsCommonClientConfig<'_> {
    pub async fn client_config_sync_version(
        &self,
        id: AccountIdInternal,
    ) -> Result<ClientConfigSyncVersion, DataError> {
        self.db_read(move |mut cmds| cmds.common().client_config().client_config_sync_version(id))
            .await
            .into_error()
    }

    /// Get the Apple App Attest key for an account by key id.
    pub async fn apple_app_attest_key(
        &self,
        id: AccountIdInternal,
        key_id: &[u8],
    ) -> Result<Option<AppleAppAttestKey>, DataError> {
        let key_id = key_id.to_vec();
        self.db_read(move |mut cmds| {
            cmds.common()
                .client_config()
                .apple_app_attest_key(id, &key_id)
        })
        .await
        .into_error()
    }

    /// Check if an Apple App Attest key exists for an account.
    pub async fn apple_app_attest_key_exists(
        &self,
        id: AccountIdInternal,
        key_id: &[u8],
    ) -> Result<bool, DataError> {
        let key_id = key_id.to_vec();
        self.db_read(move |mut cmds| {
            cmds.common()
                .client_config()
                .apple_app_attest_key_exists(id, &key_id)
        })
        .await
        .into_error()
    }

    pub async fn login_session_info(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<LoginSessionInfo>, DataError> {
        self.db_read(move |mut cmds| cmds.common().client_config().login_session_info(id))
            .await
            .into_error()
    }

    pub async fn client_login_session_platform(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<ClientType>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common()
                .client_config()
                .client_login_session_platform(id)
        })
        .await
        .into_error()
    }

    pub async fn client_language(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<ClientLanguage>, DataError> {
        self.db_read(move |mut cmds| cmds.common().client_config().client_language(id))
            .await
            .into_error()
    }
}
