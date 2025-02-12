use database_account::current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount};
use model::AccountIdInternal;
use model_account::AccountReportContent;
use server_data::{
    define_cmd_wrapper_write,
    read::DbRead,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
    DataError,
};

define_cmd_wrapper_write!(WriteCommandsAccountReport);

impl WriteCommandsAccountReport<'_> {
    pub async fn process_report(
        &self,
        moderator_id: AccountIdInternal,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        content: AccountReportContent,
    ) -> Result<(), DataError> {
        let current_report = self
            .db_read(move |mut cmds| cmds.account().report().get_report(creator, target))
            .await?;
        if current_report.content != content {
            return Err(DataError::NotAllowed.report());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.account_admin()
                .report()
                .mark_report_done(moderator_id, creator, target)?;
            Ok(())
        })?;

        Ok(())
    }
}
