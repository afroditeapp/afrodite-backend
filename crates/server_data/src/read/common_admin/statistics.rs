use database::current::read::GetDbReadCommandsCommon;
use model::{AccountIdInternal, GetApiUsageStatisticsResult, GetApiUsageStatisticsSettings, GetIpAddressStatisticsResult};
use crate::{
    define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError,
};

define_cmd_wrapper_read!(ReadCommandsCommonAdminStatistics);

impl ReadCommandsCommonAdminStatistics<'_> {
    pub async fn get_api_usage_statistics(
        &self,
        account: AccountIdInternal,
        settings: GetApiUsageStatisticsSettings,
    ) -> Result<GetApiUsageStatisticsResult, DataError> {
        self.db_read(move |mut cmds| cmds.common_admin().statistics().api_usage_statistics(account, settings))
            .await
            .into_error()
    }

    pub async fn get_ip_address_statistics(
        &self,
        account: AccountIdInternal,
    ) -> Result<GetIpAddressStatisticsResult, DataError> {
        self.db_read(move |mut cmds| cmds.common_admin().statistics().ip_address_statistics(account))
            .await
            .into_error()
    }
}
