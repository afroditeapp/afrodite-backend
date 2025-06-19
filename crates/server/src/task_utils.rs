use server_api::{
    DataError, S,
    app::{ApiUsageTrackerProvider, ClientVersionTrackerProvider, IpAddressUsageTrackerProvider},
    db_write_raw,
};
use server_data::{app::WriteData, result::Result, write::GetWriteCommandsCommon};
use server_data_account::write::GetWriteCommandsAccount;

pub struct TaskUtils;

impl TaskUtils {
    pub async fn save_client_version_statistics(state: &S) -> Result<(), DataError> {
        let statistics = state
            .client_version_tracker()
            .get_current_state_and_reset()
            .await;

        db_write_raw!(state, move |cmds| {
            cmds.account_admin_history()
                .save_client_version_statistics(statistics)
                .await
        })
        .await?;

        Ok(())
    }

    pub async fn save_api_usage_statistics(state: &S) -> Result<(), DataError> {
        let statistics = state
            .api_usage_tracker()
            .get_current_state_and_reset()
            .await;

        db_write_raw!(state, move |cmds| {
            cmds.common_admin()
                .statistics()
                .save_api_usage_data(statistics)
                .await
        })
        .await?;

        Ok(())
    }

    pub async fn save_ip_address_statistics(state: &S) -> Result<(), DataError> {
        let statistics = state
            .ip_address_usage_tracker()
            .get_current_state_and_reset()
            .await;

        db_write_raw!(state, move |cmds| {
            cmds.common_admin()
                .statistics()
                .save_ip_address_data(statistics)
                .await
        })
        .await?;

        Ok(())
    }
}
