
use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use model::{AccountIdInternal, ReportContent, ReportTypeNumber};

use crate::{
    define_cmd_wrapper_write, read::DbRead, result::{Result, WrappedContextExt}, write::db_transaction, DataError
};

use crate::write::DbTransaction;

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
        let current_report = self
            .db_read(move |mut cmds| cmds.common_admin().report().get_detailed_report(creator, target, report_type))
            .await?;
        if current_report.map(|v| v.content) != Some(content) {
            return Err(DataError::NotAllowed.report());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.common_admin()
                .report()
                .mark_report_done(moderator_id, creator, target, report_type)?;
            Ok(())
        })?;

        Ok(())
    }
}
