//! Synchronous write commands combining cache and database operations.

use account::WriteCommandsAccount;
use account_admin::WriteCommandsAccountAdmin;
use chat::WriteCommandsChatUtils;
use server_data::db_manager::WriteAccessProvider;

pub mod account;
pub mod account_admin;
pub mod chat;

pub trait GetWriteCommandsAccount<'a> {
    fn account(self) -> WriteCommandsAccount<'a>;
    fn account_admin(self) -> WriteCommandsAccountAdmin<'a>;
    fn account_chat_utils(self) -> WriteCommandsChatUtils<'a>;
}

impl<'a, C: WriteAccessProvider<'a>> GetWriteCommandsAccount<'a> for C {
    fn account(self) -> WriteCommandsAccount<'a> {
        WriteCommandsAccount::new(self.handle())
    }

    fn account_admin(self) -> WriteCommandsAccountAdmin<'a> {
        WriteCommandsAccountAdmin::new(self.handle())
    }

    fn account_chat_utils(self) -> WriteCommandsChatUtils<'a> {
        WriteCommandsChatUtils::new(self.handle())
    }
}
