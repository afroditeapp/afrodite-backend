use database::current::write::GetDbWriteCommandsCommon;
use model::{AccountIdInternal, ClientLanguage, ClientType};

use crate::{
    DataError, define_cmd_wrapper_write,
    result::Result,
    write::{DbTransaction, db_transaction},
};

define_cmd_wrapper_write!(WriteCommandsCommonClientConfig);

impl WriteCommandsCommonClientConfig<'_> {
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
