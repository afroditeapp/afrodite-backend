use database::history::read::GetDbHistoryReadCommandsCommon;
use model::{GetIpCountryStatisticsResult, GetIpCountryStatisticsSettings};
use server_common::data::IntoDataError;

use crate::{
    DataError, db_manager::InternalReading, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsCommonHistory);

impl ReadCommandsCommonHistory<'_> {
    pub async fn backup_history_database(&self, file_name: String) -> Result<(), DataError> {
        self.db_read_history_raw_no_transaction(|mut db| {
            db.common_history().backup_history_database(file_name)
        })
        .await
        .into_error()
    }

    pub async fn ip_country_statistics(
        &self,
        settings: GetIpCountryStatisticsSettings,
    ) -> Result<GetIpCountryStatisticsResult, DataError> {
        self.db_read_history(move |mut cmds| {
            cmds.common_history()
                .statistics()
                .ip_country_statistics(settings)
        })
        .await
        .into_error()
    }
}
