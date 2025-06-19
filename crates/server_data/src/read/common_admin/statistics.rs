use std::sync::Arc;

use database::current::read::GetDbReadCommandsCommon;
use model::{
    AccountIdInternal, GetApiUsageStatisticsResult, GetApiUsageStatisticsSettings,
    GetIpAddressStatisticsResult,
};
use simple_backend::maxmind_db::IpDb;

use crate::{
    DataError, IntoDataError, db_manager::InternalReading, define_cmd_wrapper_read, read::DbRead,
    result::Result,
};

define_cmd_wrapper_read!(ReadCommandsCommonAdminStatistics);

impl ReadCommandsCommonAdminStatistics<'_> {
    pub async fn get_api_usage_statistics(
        &self,
        account: AccountIdInternal,
        settings: GetApiUsageStatisticsSettings,
    ) -> Result<GetApiUsageStatisticsResult, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common_admin()
                .statistics()
                .api_usage_statistics(account, settings)
        })
        .await
        .into_error()
    }

    pub async fn get_ip_address_statistics(
        &self,
        account: AccountIdInternal,
        ip_db: Option<Arc<IpDb>>,
    ) -> Result<GetIpAddressStatisticsResult, DataError> {
        let config = self.config_arc();
        self.db_read(move |mut cmds| {
            cmds.common_admin()
                .statistics()
                .ip_address_statistics(account, config, ip_db)
        })
        .await
        .into_error()
    }
}
