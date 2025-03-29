use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use database_account::current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount};
use model::EventToClientInternal;
use model_account::AccountIdInternal;
use server_data::{
    db_manager::InternalWriting, define_cmd_wrapper_write, file::FileWrite, read::DbRead, result::Result, write::{DbTransaction, GetWriteCommandsCommon}, DataError
};

use crate::write::GetWriteCommandsAccount;

define_cmd_wrapper_write!(WriteCommandsAccountDelete);

impl WriteCommandsAccountDelete<'_> {
    pub async fn set_account_deletion_request_state(
        &self,
        id: AccountIdInternal,
        value: bool,
    ) -> Result<(), DataError> {
        let (deletion_requested, current_account) = self.db_read(move |mut cmds| {
            let deletion_requested = cmds.account().delete().account_deletion_requested(id)?;
            let current_account = cmds.common().account(id)?;
            Ok((deletion_requested, current_account))
        }).await?;
        if value == deletion_requested.is_some() {
            // Already in correct state
            return Ok(());
        }
        let a = current_account.clone();
        let new_account = db_transaction!(self, move |mut cmds| {
            let a = cmds.common()
                .state()
                .update_syncable_account_data(id, a, move |state_container, _, visibility| {
                    state_container.set_pending_deletion(value);
                    if value {
                        visibility.change_to_private_or_pending_private();
                    }
                    Ok(())
                })?;

            cmds.account().delete().set_account_deletion_request_state(id, value)?;

            Ok(a)
        })?;

        self.handle()
            .common()
            .internal_handle_new_account_data_after_db_modification(
                id,
                &current_account,
                &new_account,
            )
            .await?;

        self.events()
            .send_connected_event(
                id.uuid,
                EventToClientInternal::AccountStateChanged,
            )
            .await?;

        Ok(())
    }

    /// NOTE: IpAddressUsageTracker also ApiUsageTracker
    /// in RAM data for account but that is processed hourly and that
    /// deletes the data for non existing accounts.
    pub async fn delete_account(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        self.handle().common().logout(id).await?;

        // Delete account from location index
        self.handle().account().update_syncable_account_data(id, None, |_, _, visibility| {
            visibility.change_to_private_or_pending_private();
            Ok(())
        }).await?;

        db_transaction!(self, move |mut cmds| {
            cmds.account().delete().delete_account(id)
        })?;

        self.cache().delete_account_which_is_logged_out(id.as_id()).await;

        self.files().account_dir(id.as_id()).delete_if_exists().await?;

        Ok(())
    }
}
