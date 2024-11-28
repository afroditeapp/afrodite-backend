//! Synchronous write commands combining cache and database operations.

use account::WriteCommandsAccount;
use account_admin::WriteCommandsAccountAdmin;
use server_data::db_manager::WriteAccessProvider;

pub mod account;
pub mod account_admin;

pub trait GetWriteCommandsAccount<'a> {
    fn account(self) -> WriteCommandsAccount<'a>;
    fn account_admin(self) -> WriteCommandsAccountAdmin<'a>;
}

impl<'a, C: WriteAccessProvider<'a>> GetWriteCommandsAccount<'a> for C {
    fn account(self) -> WriteCommandsAccount<'a> {
        WriteCommandsAccount::new(self.handle())
    }

    fn account_admin(self) -> WriteCommandsAccountAdmin<'a> {
        WriteCommandsAccountAdmin::new(self.handle())
    }
}
