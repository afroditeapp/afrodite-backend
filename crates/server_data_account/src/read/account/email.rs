use model_account::{AccountEmailSendingStateRaw, AccountIdInternal};
use server_data::{
    define_cmd_wrapper, result::Result, DataError
};

use crate::read::DbReadAccount;

define_cmd_wrapper!(ReadCommandsAccountEmail);

impl<C: DbReadAccount> ReadCommandsAccountEmail<C> {
    pub async fn email_state(
        &self,
        id: AccountIdInternal,
    ) -> Result<AccountEmailSendingStateRaw, DataError> {
        let state = self
            .db_read(move |mut cmds| cmds.account().email().email_sending_states(id))
            .await?;
        Ok(state)
    }
}
