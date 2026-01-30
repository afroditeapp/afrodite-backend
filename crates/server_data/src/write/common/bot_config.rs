use database::current::write::GetDbWriteCommandsCommon;
use model::BotConfig;

use crate::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsCommonBotConfig);

impl WriteCommandsCommonBotConfig<'_> {
    pub async fn upsert_bot_config(&self, config: &BotConfig) -> Result<(), DataError> {
        let config = config.clone();
        db_transaction!(self, move |mut cmds| {
            cmds.common().bot_config().upsert_bot_config(&config)
        })
    }
}
