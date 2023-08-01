use super::{WriteCommandRunnerHandle, ResultSender, WriteCommandRunner, SendBack};






use error_stack::Result;




use crate::{
    api::{
        model::{
            Account, AccountIdInternal, AccountIdLight, AccountSetup, SignInWithInfo,
        },
    },
    server::data::{DatabaseError},
};





/// Synchronized write commands.
#[derive(Debug)]
pub enum AccountWriteCommand {
    Register {
        s: ResultSender<AccountIdInternal>,
        sign_in_with_info: SignInWithInfo,
        account_id: AccountIdLight,
    },
    UpdateAccount {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        account: Account,
    },
    UpdateAccountSetup {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        account_setup: AccountSetup,
    },
}

#[derive(Debug, Clone)]
pub struct AccountWriteCommandRunnerHandle<'a> {
    pub handle: &'a WriteCommandRunnerHandle,
}

impl AccountWriteCommandRunnerHandle<'_> {
    pub async fn register(
        &self,
        account_id: AccountIdLight,
        sign_in_with_info: SignInWithInfo,
    ) -> Result<AccountIdInternal, DatabaseError> {
        self.handle.send_event(|s| AccountWriteCommand::Register {
            s,
            sign_in_with_info,
            account_id,
        })
        .await
    }

    pub async fn update_account(
        &self,
        account_id: AccountIdInternal,
        account: Account,
    ) -> Result<(), DatabaseError> {
        self.handle.send_event(|s| AccountWriteCommand::UpdateAccount {
            s,
            account_id,
            account,
        })
        .await
    }

    pub async fn update_account_setup(
        &self,
        account_id: AccountIdInternal,
        account_setup: AccountSetup,
    ) -> Result<(), DatabaseError> {
        self.handle.send_event(|s| AccountWriteCommand::UpdateAccountSetup {
            s,
            account_id,
            account_setup,
        })
        .await
    }
}



impl WriteCommandRunner {
    pub async fn handle_account_cmd(&self, cmd: AccountWriteCommand) {
        match cmd {
            AccountWriteCommand::Register {
                s,
                sign_in_with_info,
                account_id,
            } => self
                .write_handle
                .register(account_id, sign_in_with_info, &self.config)
                .await
                .send(s),
            AccountWriteCommand::UpdateAccount {
                s,
                account_id,
                account,
            } => self.write().update_data(account_id, &account).await.send(s),
            AccountWriteCommand::UpdateAccountSetup {
                s,
                account_id,
                account_setup,
            } => self
                .write()
                .update_data(account_id, &account_setup)
                .await
                .send(s),
        }
    }
}
