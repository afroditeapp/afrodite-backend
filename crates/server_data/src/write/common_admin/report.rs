use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use model::{AccountIdInternal, ProcessReport, ReportContent, ReportType, ReportTypeInternal};
use simple_backend_utils::IntoReportFromString;

use crate::{
    DataError, IntoDataError,
    app::GetConfig,
    db_transaction, define_cmd_wrapper_write,
    id::ToAccountIdInternal,
    read::DbRead,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsCommonAdminReport);

impl WriteCommandsCommonAdminReport<'_> {
    async fn process_single_report(
        &self,
        moderator_id: AccountIdInternal,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        report_type: ReportType,
        content: ReportContent,
        valid: bool,
    ) -> Result<(), DataError> {
        let report_type = TryInto::<ReportTypeInternal>::try_into(Into::<i16>::into(report_type.n))
            .into_error_string(DataError::NotAllowed)?;

        let current_reports = self
            .db_read(move |mut cmds| {
                cmds.common()
                    .report()
                    .get_all_detailed_reports(creator, target, report_type)
            })
            .await?;

        let matching_report = current_reports.iter().find(|v| v.report.content == content);
        if let Some(report) = matching_report {
            let id = report.id;
            db_transaction!(self, move |mut cmds| {
                cmds.common_admin()
                    .report()
                    .mark_report_processed(moderator_id, id, valid)?;
                Ok(())
            })?;
            Ok(())
        } else {
            Err(DataError::NotAllowed.report())
        }
    }

    pub async fn process_reports_and_get_report_spammers(
        &self,
        moderator_id: AccountIdInternal,
        reports: Vec<ProcessReport>,
    ) -> Result<Vec<AccountIdInternal>, DataError> {
        let mut creators_set: std::collections::HashSet<AccountIdInternal> =
            std::collections::HashSet::new();
        for report in &reports {
            let creator = self.to_account_id_internal(report.creator).await?;
            creators_set.insert(creator);
            let target = self.to_account_id_internal(report.target).await?;
            self.process_single_report(
                moderator_id,
                creator,
                target,
                report.report_type,
                report.content.clone(),
                report.valid,
            )
            .await?;
        }

        let threshold = self
            .config()
            .limits_common()
            .auto_ban_spam_reporters_invalid_report_threshold;

        let mut reporters_to_ban = Vec::new();
        for creator in &creators_set {
            let creator_db_id = *creator.as_db_id();
            let count = self
                .db_read(move |mut cmds| {
                    cmds.common_admin()
                        .report()
                        .get_invalid_report_count_for_report_spammer_detection(creator_db_id)
                })
                .await
                .into_error()?;

            if count >= threshold as i64 {
                reporters_to_ban.push(*creator);
            }
        }

        Ok(reporters_to_ban)
    }
}
