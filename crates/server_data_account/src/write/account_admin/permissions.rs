use model::Permissions;
use model_account::AccountIdInternal;
use server_data::{DataError, define_cmd_wrapper_write, result::Result};

use crate::write::GetWriteCommandsAccount;

define_cmd_wrapper_write!(WriteCommandsAccountPermissionsAdmin);

impl WriteCommandsAccountPermissionsAdmin<'_> {
    pub async fn set_permissions(
        &self,
        id: AccountIdInternal,
        permissions: Permissions,
    ) -> Result<(), DataError> {
        self.handle()
            .account()
            .update_syncable_account_data(id, None, |account| {
                account.permissions = permissions;
                Ok(())
            })
            .await?;

        Ok(())
    }
}
