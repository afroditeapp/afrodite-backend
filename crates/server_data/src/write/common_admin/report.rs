use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use model::{AccountIdInternal, ReportContent, ReportTypeNumber, ReportTypeNumberInternal};
use simple_backend_utils::IntoReportFromString;

use crate::{
    DataError,
    app::GetConfig,
    db_transaction, define_cmd_wrapper_write,
    read::DbRead,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsCommonAdminReport);

impl WriteCommandsCommonAdminReport<'_> {
    pub async fn process_report(
        &self,
        moderator_id: AccountIdInternal,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        report_type: ReportTypeNumber,
        content: ReportContent,
    ) -> Result<(), DataError> {
        let report_type = TryInto::<ReportTypeNumberInternal>::try_into(report_type)
            .into_error_string(DataError::NotAllowed)?;

        let components = self.config().components();
        let current_reports = self
            .db_read(move |mut cmds| {
                cmds.common().report().get_all_detailed_reports(
                    creator,
                    target,
                    report_type,
                    components,
                )
            })
            .await?;

        let matching_report = current_reports.iter().find(|v| v.report.content == content);
        if let Some(report) = matching_report {
            let id = report.id;
            db_transaction!(self, move |mut cmds| {
                cmds.common_admin()
                    .report()
                    .mark_report_done(moderator_id, id)?;
                Ok(())
            })?;
            Ok(())
        } else {
            Err(DataError::NotAllowed.report())
        }
    }
}
