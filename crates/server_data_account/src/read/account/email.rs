use model_account::{AccountEmailSendingStateRaw, AccountIdInternal};
use server_data::{
    define_server_data_read_commands, read::ReadCommandsProvider, result::Result, DataError,
};

define_server_data_read_commands!(ReadCommandsAccountEmail);
define_db_read_command!(ReadCommandsAccountEmail);

impl<C: ReadCommandsProvider> ReadCommandsAccountEmail<C> {
    pub async fn email_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AccountEmailSendingStateRaw, DataError> {
        let state = self
            .db_read(move |mut cmds| cmds.account().email().email_sending_states(id))
            .await?;
        Ok(state)
    }
}
