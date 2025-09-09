use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use database_account::current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount};
use model::{
    AccountIdInternal, CustomReportId, CustomReportType, ReportTypeNumber,
    ReportTypeNumberInternal, UpdateReportResult,
};
use server_data::{
    DataError,
    app::GetConfig,
    db_transaction, define_cmd_wrapper_write,
    read::DbRead,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
};
use tracing::info;

define_cmd_wrapper_write!(WriteCommandsAccountReport);

impl WriteCommandsAccountReport<'_> {
    /// Reports custom report with empty content if not previously reported.
    pub async fn report_custom_report_empty(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        custom_report_id: CustomReportId,
    ) -> Result<UpdateReportResult, DataError> {
        let custom_report_type_number = custom_report_id
            .to_report_type_number_value()
            .map_err(|e| DataError::NotAllowed.report().attach_printable(e))?;

        let custom_report_type = self
            .config()
            .custom_reports()
            .and_then(|v| v.index_with_id(custom_report_id))
            .map(|r| r.report_type);
        if custom_report_type != Some(CustomReportType::Empty) {
            return Err(DataError::NotAllowed.report());
        }

        let reports = self
            .db_read(move |mut cmds| {
                cmds.common().report().get_all_detailed_reports(
                    creator,
                    target,
                    ReportTypeNumberInternal::CustomReport(custom_report_type_number),
                )
            })
            .await?;
        if reports.len() >= ReportTypeNumber::MAX_COUNT {
            return Ok(UpdateReportResult::too_many_reports());
        }

        if !reports.is_empty() {
            // Already reported
            return Ok(UpdateReportResult::success());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.account().report().insert_custom_report_empty(
                creator,
                target,
                custom_report_type_number,
            )?;
            Ok(())
        })?;

        Ok(UpdateReportResult::success())
    }

    /// Updates the custom reports sha256 and related sync version for it for every
    /// account if needed.
    pub async fn update_custom_reports_sha256_and_sync_versions(
        &self,
        sha256: String,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let current_hash = cmds.read().account().report().custom_reports_hash()?;

            if current_hash.as_deref() != Some(&sha256) {
                info!(
                    "Custom reports file hash changed from {:?} to {:?}",
                    current_hash,
                    Some(&sha256)
                );

                cmds.account()
                    .report()
                    .upsert_custom_reports_file_hash(&sha256)?;

                cmds.common()
                    .client_config()
                    .increment_client_config_sync_version_for_every_account()?;
            }

            Ok(())
        })
    }
}
