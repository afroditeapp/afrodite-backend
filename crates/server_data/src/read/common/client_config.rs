use database::current::read::GetDbReadCommandsCommon;
use model::{AccountIdInternal, ClientConfigSyncVersion, ClientLanguage, ClientType};

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
