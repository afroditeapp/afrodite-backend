use database_account::current::read::GetDbReadCommandsAccount;
use model_account::{AccountIdInternal, AssociationMembership};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsAccountAssociation);

impl ReadCommandsAccountAssociation<'_> {
    pub async fn get_own_entry(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<AssociationMembership>, DataError> {
        self.db_read(move |mut cmds| cmds.account().association().get_own_entry(id))
            .await
            .into_error()
    }
}
