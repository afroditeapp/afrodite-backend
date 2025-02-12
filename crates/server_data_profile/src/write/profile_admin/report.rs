use database_profile::current::{read::GetDbReadCommandsProfile, write::GetDbWriteCommandsProfile};
use model_profile::{AccountIdInternal, ProfileReportContent};
use server_data::{
    define_cmd_wrapper_write,
    read::DbRead,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
    DataError,
};

define_cmd_wrapper_write!(WriteCommandsProfileReport);

impl WriteCommandsProfileReport<'_> {
    pub async fn process_report(
        &self,
        moderator_id: AccountIdInternal,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        content: ProfileReportContent,
    ) -> Result<(), DataError> {
        let current_report = self
            .db_read(move |mut cmds| cmds.profile().report().get_report(creator, target))
            .await?;
        if current_report.content.profile_text != content.profile_text {
            return Err(DataError::NotAllowed.report());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.profile_admin()
                .report()
                .mark_report_done(moderator_id, creator, target)?;
            Ok(())
        })?;

        Ok(())
    }
}
