use database_media::current::{read::GetDbReadCommandsMedia, write::GetDbWriteCommandsMedia};
use model_media::{AccountIdInternal, MediaReportContent};
use server_data::{
    define_cmd_wrapper_write,
    read::DbRead,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
    DataError,
};

define_cmd_wrapper_write!(WriteCommandsMediaReport);

impl WriteCommandsMediaReport<'_> {
    pub async fn process_report(
        &self,
        moderator_id: AccountIdInternal,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        content: MediaReportContent,
    ) -> Result<(), DataError> {
        let current_report = self
            .db_read(move |mut cmds| cmds.media().report().get_report(creator, target))
            .await?;
        if current_report.content.profile_content != content.profile_content {
            return Err(DataError::NotAllowed.report());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.media_admin()
                .report()
                .mark_report_done(moderator_id, creator, target)?;
            Ok(())
        })?;

        Ok(())
    }
}
