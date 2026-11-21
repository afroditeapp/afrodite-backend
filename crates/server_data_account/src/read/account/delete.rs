use database_account::current::read::GetDbReadCommandsAccount;
use model_account::{AccountIdInternal, GetAccountDeletionRequestResult};
use server_data::{
    DataError, IntoDataError, db_manager::InternalReading, define_cmd_wrapper_read, read::DbRead,
    result::Result,
};

define_cmd_wrapper_read!(ReadCommandsAccountDelete);

impl ReadCommandsAccountDelete<'_> {
    /// Returns whether the deletion request was initiated automatically
    /// e.g. for inactive account.
    pub async fn account_deletion_requested_is_automatic(
        &self,
        id: AccountIdInternal,
    ) -> Result<bool, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account()
                .delete()
                .account_deletion_requested_is_automatic(id)
        })
        .await
        .into_error()
    }

    /// Returns deletion allowed time for account deletion request.
    pub async fn account_deletion_state(
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
