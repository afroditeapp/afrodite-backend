use database::current::read::GetDbReadCommandsCommon;
use model::BackendConfig;
use server_common::data::IntoDataError;

use super::DbRead;
use crate::{DataError, define_cmd_wrapper_read, result::Result};

define_cmd_wrapper_read!(ReadCommandsCommonBotConfig);

impl ReadCommandsCommonBotConfig<'_> {
    pub async fn bot_config(&self) -> Result<Option<BackendConfig>, DataError> {
        self.db_read(move |mut cmds| cmds.common().bot_config().bot_config())
            .await
            .into_error()
    }
}
