//! Synchronous write commands combining cache and database operations.

use account::WriteCommandsAccount;
use account_admin::WriteCommandsAccountAdmin;
use chat::WriteCommandsChatUtils;
use server_data::db_manager::WriteAccessProvider;

pub mod account;
pub mod account_admin;
pub mod chat;

pub trait GetWriteCommandsAccount {
    fn account(&self) -> WriteCommandsAccount<'_>;
    fn account_admin(&self) -> WriteCommandsAccountAdmin<'_>;
    fn account_chat_utils(&self) -> WriteCommandsChatUtils<'_>;
}

impl<C: WriteAccessProvider> GetWriteCommandsAccount for C {
    fn account(&self) -> WriteCommandsAccount<'_> {
        WriteCommandsAccount::new(self.handle())
    }

    fn account_admin(&self) -> WriteCommandsAccountAdmin<'_> {
        WriteCommandsAccountAdmin::new(self.handle())
    }

    fn account_chat_utils(&self) -> WriteCommandsChatUtils<'_> {
        WriteCommandsChatUtils::new(self.handle())
    }
}
