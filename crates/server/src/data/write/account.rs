use model::{
    AccountData, AccountIdInternal, AccountInternal, AccountSetup, Capabilities, SharedState,
};

use super::db_transaction;
use crate::{data::DataError, result::Result};

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
        db_transaction!(self, move |mut cmds| {
            if let Some(state) = state_copy {
                cmds.common().state().shared_state(id, state)?;
            }
            if let Some(capabilities) = capabilities_copy {
                cmds.common()
                    .state()
                    .account_capabilities(id, capabilities)?;
            }
            Ok(())
        })?;

        self.write_cache(id, |cache| {
            if let Some(state) = shared_state {
                cache.shared_state = state;
            }
            if let Some(capabilities) = capabilities {
                cache.capabilities = capabilities;
            }
            Ok(())
        })
        .await?;

        Ok(())
    }

    pub async fn account_setup(
        &self,
        id: AccountIdInternal,
        account_setup: AccountSetup,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().data().account_setup(id, &account_setup)
        })
    }

    pub async fn account_data(
        &self,
        id: AccountIdInternal,
        account_data: AccountData,
    ) -> Result<(), DataError> {
        let internal = AccountInternal {
            email: account_data.email,
        };

        db_transaction!(self, move |mut cmds| {
            cmds.account().data().account(id, &internal)
        })
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
