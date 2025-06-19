use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use database_account::current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount};
use model::{
    AccountIdInternal, CustomReportId, CustomReportType, ReportTypeNumber,
    ReportTypeNumberInternal, UpdateReportResult,
};
use server_data::{
    DataError,
    app::GetConfig,
    define_cmd_wrapper_write,
    read::DbRead,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
};
use tracing::info;

define_cmd_wrapper_write!(WriteCommandsAccountReport);

impl WriteCommandsAccountReport<'_> {
    /// Reports custom report with boolean value if not previously reported with
    /// the same value.
    pub async fn report_custom_report_boolean(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        custom_report_id: CustomReportId,
        value: bool,
    ) -> Result<UpdateReportResult, DataError> {
        let custom_report_type_number = custom_report_id
            .to_report_type_number_value()
            .map_err(|e| DataError::NotAllowed.report().attach_printable(e))?;

        let custom_report_type = self
            .config()
            .custom_reports()
            .and_then(|v| v.index_with_id(custom_report_id))
            .map(|r| r.report_type);
        if custom_report_type != Some(CustomReportType::Boolean) {
            return Err(DataError::NotAllowed.report());
        }

        let components = self.config().components();
        let reports = self
            .db_read(move |mut cmds| {
                cmds.common().report().get_all_detailed_reports(
                    creator,
                    target,
                    ReportTypeNumberInternal::CustomReport(custom_report_type_number),
                    components,
                )
            })
            .await?;
        if reports.len() >= ReportTypeNumber::MAX_COUNT {
            return Ok(UpdateReportResult::too_many_reports());
        }

        let report_with_same_value = reports.iter().find(|v| {
            v.report
                .content
                .custom_report
                .as_ref()
                .and_then(|v| v.boolean_value)
                == Some(value)
        });
        if report_with_same_value.is_some() {
            // Already reported
            return Ok(UpdateReportResult::success());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.account().report().insert_custom_report_boolean(
                creator,
                target,
                custom_report_type_number,
                value,
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
