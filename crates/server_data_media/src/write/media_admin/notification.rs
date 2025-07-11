use database_media::current::write::GetDbWriteCommandsMedia;
use model_media::AccountIdInternal;
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsMediaAdminNotification);

impl WriteCommandsMediaAdminNotification<'_> {
    pub async fn show_media_content_accepted_notification(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media_admin()
                .notification()
                .show_media_content_accepted_notification(id)?;
            Ok(())
        })?;

        Ok(())
    }

    pub async fn show_media_content_rejected_notification(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media_admin()
                .notification()
                .show_media_content_rejected_notification(id)?;
            Ok(())
        })?;

        Ok(())
    }
}
