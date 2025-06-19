use database_account::current::read::GetDbReadCommandsAccount;
use model_account::{AccountIdInternal, GetAccountDeletionRequestResult};
use server_data::{
    DataError, db_manager::InternalReading, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsAccountDelete);

impl ReadCommandsAccountDelete<'_> {
    pub async fn account_deleteion_state(
        &self,
        id: AccountIdInternal,
    ) -> Result<GetAccountDeletionRequestResult, DataError> {
        let deletion_requested_time = self
            .db_read(move |mut cmds| cmds.account().delete().account_deletion_requested(id))
            .await?;

        let deletion_wait_time = self
            .config()
            .limits_account()
            .account_deletion_wait_duration;
        let automatic_deletion_allowed =
            deletion_requested_time.map(|time| time.add_seconds(deletion_wait_time.seconds));

        Ok(GetAccountDeletionRequestResult {
            automatic_deletion_allowed,
        })
    }
}
