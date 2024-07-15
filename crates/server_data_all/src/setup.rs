
use model::{
    AccountIdInternal, SetAccountSetup, SetProfileSetup,
};
use server_data::{
    define_server_data_write_commands, result::Result, write::WriteCommandsProvider, DataError
};

define_server_data_write_commands!(SetSetupDataCmd);
define_db_transaction_command!(SetSetupDataCmd);

impl<C: WriteCommandsProvider> SetSetupDataCmd<C> {
    /// Account component sets setup data to all components.
    pub async fn set_account_and_profile_setup_data(
        &self,
        id: AccountIdInternal,
        account_setup: SetAccountSetup,
        profile_setup: SetProfileSetup,
    ) -> Result<(), DataError> {
        let is_account_component_enabled = self.config().components().account;
        let is_profile_component_enabled = self.config().components().profile;
        db_transaction!(self, move |mut cmds| {
            if is_account_component_enabled {
                cmds.account().data().account_setup(id, &account_setup)?;
            }

            if is_profile_component_enabled {
                cmds.profile().setup().profile_setup(id, &profile_setup)?;
            }

            Ok(())
        })?;

        Ok(())
    }
}
