use database::current::write::{CurrentSyncWriteCommands};
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, AccountSetup, Capabilities, AccountData, AccountInternal, SharedState};

use crate::data::DataError;

define_write_commands!(WriteCommandsAccount);

impl WriteCommandsAccount<'_> {
    /// Remember to sync another servers if you use this method
    pub async fn update_account_state_and_capabilities(
        &self,
        id: AccountIdInternal,
        shared_state: Option<SharedState>,
        capabilities: Option<Capabilities>,
    ) -> Result<(), DataError> {
        let state_copy = shared_state.clone();
        let capabilities_copy = capabilities.clone();
        self.db_transaction(move |cmds| {
            let mut cmds = CurrentSyncWriteCommands::new(cmds);

            if let Some(state) = state_copy {
                cmds.common().shared_state(id, state)?;
            }
            if let Some(capabilities) = capabilities_copy {
                cmds.common().account_capabilities(id, capabilities)?;
            }
            Ok(())
        }).await?;

        self.write_cache(id, |cache| {
            if let Some(state) = shared_state {
                cache.shared_state = state;
            }
            if let Some(capabilities) = capabilities {
                cache.capabilities = capabilities;
            }
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

    pub async fn account_data(
        &self,
        id: AccountIdInternal,
        account_data: AccountData,
    ) -> Result<(), DataError> {
        let internal = AccountInternal {
            email: account_data.email,
        };

        self.db_write(move |cmds| cmds.into_account().account(id, &internal))
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
