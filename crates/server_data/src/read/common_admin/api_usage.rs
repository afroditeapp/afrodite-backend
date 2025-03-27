use database::current::read::GetDbReadCommandsCommon;
use model::{AccountIdInternal, GetApiUsageStatisticsResult, GetApiUsageStatisticsSettings};
use crate::{
    define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError,
};

define_cmd_wrapper_read!(ReadCommandsCommonAdminApiUsage);

impl ReadCommandsCommonAdminApiUsage<'_> {
    pub async fn get_api_usage_statistics(
        &self,
        account: AccountIdInternal,
        settings: GetApiUsageStatisticsSettings,
    ) -> Result<GetApiUsageStatisticsResult, DataError> {
        self.db_read(move |mut cmds| cmds.common_admin().api_usage().api_usage_statistics(account, settings))
            .await
            .into_error()
    }
}
