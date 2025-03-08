
use database::history::read::GetDbHistoryReadCommandsCommon;
use server_common::data::IntoDataError;

use crate::{
    db_manager::InternalReading, define_cmd_wrapper_read, result::Result, DataError
};

define_cmd_wrapper_read!(ReadCommandsCommonHistory);

impl ReadCommandsCommonHistory<'_> {
    pub async fn backup_history_database(
        &self,
        file_name: String,
    ) -> Result<(), DataError> {
        self.db_read_history_raw_no_transaction(|mut db| db.common_history().backup_history_database(file_name))
            .await
            .into_error()
    }
}
