use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use database_account::current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount};
use model::{Account, ProfileVisibility};
use model_account::AccountIdInternal;
use server_data::{
    db_manager::InternalWriting, define_cmd_wrapper_write, file::FileWrite, read::DbRead, result::Result, write::{DbTransaction, GetWriteCommandsCommon}, DataError
};

define_cmd_wrapper_write!(WriteCommandsAccountDelete);

impl WriteCommandsAccountDelete<'_> {
    pub async fn set_account_deletion_request_state(
        &self,
        id: AccountIdInternal,
        value: bool,
    ) -> Result<Option<Account>, DataError> {
        let (deletion_requested, current_account) = self.db_read(move |mut cmds| {
            let deletion_requested = cmds.account().delete().account_deletion_requested(id)?;
            let current_account = cmds.common().account(id)?;
            Ok((deletion_requested, current_account))
        }).await?;
        if value == deletion_requested.is_some() {
            // Already in correct state
            return Ok(None);
        }
        let a = current_account.clone();
        let new_account = db_transaction!(self, move |mut cmds| {
            let a = cmds.common()
                .state()
                .update_syncable_account_data(id, a, move |state_container, _, visibility| {
                    state_container.set_pending_deletion(value);
                    if value {
                        let new_visibility = match *visibility {
                            ProfileVisibility::Public |
                            ProfileVisibility::Private => ProfileVisibility::Private,
                            ProfileVisibility::PendingPublic |
                            ProfileVisibility::PendingPrivate => ProfileVisibility::PendingPrivate,
                        };
                        *visibility = new_visibility;
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

        Ok(Some(new_account))
    }

    pub async fn delete_account(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        self.handle().common().logout(id).await?;

        db_transaction!(self, move |mut cmds| {
            cmds.account().delete().delete_account(id)
        })?;

        self.cache().delete_account_which_is_logged_out(id.as_id()).await;

        self.files().account_dir(id.as_id()).delete_if_exists().await?;

        Ok(())
    }
}
