use database::current::write::GetDbWriteCommandsCommon;
use model::UnixTime;

use crate::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsCommonServerInfo);

impl WriteCommandsCommonServerInfo<'_> {
    pub async fn upsert_server_info(
        &self,
        version: String,
        server_start_time: UnixTime,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .server_info()
                .upsert_server_info(&version, server_start_time)
        })
    }

    pub async fn update_scheduled_tasks_start_time(
        &self,
        start_time: UnixTime,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .server_info()
                .update_scheduled_tasks_start_time(start_time)
        })
    }
}
