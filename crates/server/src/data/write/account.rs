use error_stack::Result;
use model::{Account, AccountIdInternal, AccountSetup};

use crate::data::DatabaseError;

define_write_commands!(WriteCommandsAccount);

impl WriteCommandsAccount<'_> {
    pub async fn account(
        &self,
        id: AccountIdInternal,
        account: Account,
    ) -> Result<(), DatabaseError> {
        let a = account.clone();
        self.db_write(move |cmds| cmds.into_account().account(id, &a))
            .await?;
        self.write_cache(id, |cache| {
            Ok(cache.account.as_mut().map(|data| *data.as_mut() = account))
        })
        .await?;
        Ok(())
    }

    pub async fn account_setup(
        &self,
        id: AccountIdInternal,
        account_setup: AccountSetup,
    ) -> Result<(), DatabaseError> {
        self.db_write(move |cmds| cmds.into_account().account_setup(id, &account_setup))
            .await?;
        Ok(())
    }
}
