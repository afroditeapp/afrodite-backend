use database::current::write::GetDbWriteCommandsCommon;
use model_profile::{AccountIdInternal, PendingAppNotificationInternal};
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsProfileAdminNotification);

impl WriteCommandsProfileAdminNotification<'_> {
    pub async fn show_automatic_profile_search_notification(
        &self,
        id: AccountIdInternal,
        profile_count: i64,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .notification()
                .upsert_pending_app_notification(
                    id,
                    PendingAppNotificationInternal::AutomaticProfileSearchCompleted {
                        profile_count,
                    },
                )
        })?;

        Ok(())
    }
}
