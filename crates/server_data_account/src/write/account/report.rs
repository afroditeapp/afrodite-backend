use database_account::current::write::GetDbWriteCommandsAccount;
use model::AccountIdInternal;
use model_account::AccountReportContent;
use server_data::{
    define_cmd_wrapper_write,
    result::Result,
    write::DbTransaction,
    DataError,
};

define_cmd_wrapper_write!(WriteCommandsAccountReport);

impl WriteCommandsAccountReport<'_> {
    pub async fn update_report(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        reported_content: AccountReportContent,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account()
                .report()
                .upsert_report(creator, target, reported_content)?;
            Ok(())
        })?;

        Ok(())
    }
}
