use database::current::write::{account::CurrentSyncWriteAccount, CurrentSyncWriteCommands};
use error_stack::{Result, ResultExt};
use model::{Account, AccountIdInternal, AccountSetup, Capabilities, AccountState};

use crate::data::DataError;

define_write_commands!(WriteCommandsAccount);

impl WriteCommandsAccount<'_> {
    /// Remember to sync another servers if you use this method
    pub async fn update_account_state_and_capabilities(
        &self,
        id: AccountIdInternal,
        account: Option<AccountState>,
        capabilities: Option<Capabilities>,
    ) -> Result<(), DataError> {
        let state_copy = account.clone();
        let capabilities_copy = capabilities.clone();
        self.db_transaction(move |cmds| {
            let mut cmds = CurrentSyncWriteCommands::new(cmds);

            if let Some(account) = state_copy {
                cmds.common().account_state(id, account)?;
            }
            if let Some(capabilities) = capabilities_copy {
                cmds.common().account_capabilities(id, capabilities)?;
            }
            Ok(())
        }).await?;

        self.write_cache(id, |cache| {
            cache
                .account
                .as_mut()
                .map(|data| {
                    if let Some(state) = account {
                        *data.as_mut().state_mut() = state;
                    }
                    if let Some(capabilities) = capabilities {
                        *data.as_mut().capablities_mut() = capabilities;
                    }
                });
            Ok(())
        }).await?;

        Ok(())
    }

    pub async fn account_setup(
        &self,
        id: AccountIdInternal,
        account_setup: AccountSetup,
    ) -> Result<(), DataError> {
        self.db_write(move |cmds| cmds.into_account().account_setup(id, &account_setup))
            .await?;
        Ok(())
    }

    // Remember to sync another servers if you use this method
    // pub async fn modify_capablities(&self, id: AccountIdInternal, action: impl FnOnce(&mut Capabilities)) -> Result<Capabilities, DataError> {
    //     let mut capabilities = self.db_read(move |mut cmds| cmds.common().account_capabilities(id)).await?;
    //     action(&mut capabilities);
    //     let data = capabilities.clone();
    //     self.db_write(move |mut cmds| cmds.common().account_capabilities(id, data)).await?;
    //     Ok(capabilities)
    // }
}
