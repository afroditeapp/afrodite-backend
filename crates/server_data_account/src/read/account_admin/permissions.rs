use model::Permissions;
use model_account::{AdminInfo, GetAllAdminsResult};
use server_data::{
    DataError, db_manager::InternalReading, define_cmd_wrapper_read, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsAccountPermissionsAdmin);

impl ReadCommandsAccountPermissionsAdmin<'_> {
    pub async fn all_admins(&self) -> Result<GetAllAdminsResult, DataError> {
        let mut admins = vec![];

        self.cache()
            .read_cache_for_all_accounts(|aid, entry| {
                if entry.common.account.permissions() != Permissions::default() {
                    admins.push(AdminInfo {
                        aid: aid.uuid,
                        permissions: entry.common.account.permissions(),
                    });
                }
                Ok(())
            })
            .await?;

        Ok(GetAllAdminsResult { admins })
    }
}
