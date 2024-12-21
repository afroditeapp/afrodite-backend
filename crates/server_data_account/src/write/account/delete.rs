use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use database_account::{current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount}, history::write::GetDbHistoryWriteCommandsAccount};
use model::Account;
use model_account::AccountIdInternal;
use server_data::{
    db_manager::InternalWriting, define_cmd_wrapper_write, file::FileWrite, read::DbRead, result::Result, write::{DbTransaction, GetWriteCommandsCommon}, DataError
};

// TODO(prod): Consider removing history DB as it is not possible
//             to start with a new history DB when it takes too much disk space.
//             That also increases reliability where currently both DBs are
//             modified. Probably best to move the tables which prevents the
//             previous use case to current DB.

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
                .update_syncable_account_data(id, a, move |state_container, _, _| {
                    state_container.set_pending_deletion(value);
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

        self.db_transaction_with_history(move |transaction, mut history_conn| {
            history_conn.account_history().delete_account(id)?;
            let mut current = transaction.into_conn();
            current.account().delete().delete_account(id)?;
            Ok(())
        }).await?;

        self.cache().delete_account_which_is_logged_out(id.as_id()).await;

        self.files().account_dir(id.as_id()).delete_if_exists().await?;

        Ok(())
    }
}
