use model::Permissions;
use model_account::{AdminInfo, GetAllAdminsResult};
use server_data::{
    db_manager::InternalReading, define_cmd_wrapper_read, result::Result, DataError
};

define_cmd_wrapper_read!(ReadCommandsAccountPermissionsAdmin);

impl ReadCommandsAccountPermissionsAdmin<'_> {
    pub async fn all_admins(&self) -> Result<GetAllAdminsResult, DataError> {
        let mut admins = vec![];

        self.cache().read_cache_for_all_accounts(|aid, entry| {
            if entry.common.permissions != Permissions::default() {
                admins.push(AdminInfo {
                    aid: aid.uuid,
                    permissions: entry.common.permissions.clone(),
                });
            }
            Ok(())
        }).await?;

        Ok(GetAllAdminsResult {
            admins,
        })
    }
}
