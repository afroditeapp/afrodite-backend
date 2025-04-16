use database_profile::current::write::GetDbWriteCommandsProfile;
use model_profile::AccountIdInternal;
use server_data::{
    define_cmd_wrapper_write, result::Result, write::DbTransaction, DataError
};

define_cmd_wrapper_write!(WriteCommandsProfileAdminNotification);

impl WriteCommandsProfileAdminNotification<'_> {
    pub async fn show_profile_text_accepted_notification(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile_admin()
                .notification()
                .show_profile_text_accepted_notification(id)?;
            Ok(())
        })?;

        Ok(())
    }

    pub async fn show_profile_text_rejected_notification(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile_admin()
                .notification()
                .show_profile_text_rejected_notification(id)?;
            Ok(())
        })?;

        Ok(())
    }
}
