//! Synchronous write commands combining cache and database operations.

use account::WriteCommandsAccount;
use account_admin::WriteCommandsAccountAdmin;
use server_data::write::WriteCommands;

pub mod account;
pub mod account_admin;

pub trait GetWriteCommandsAccount<'a>: Sized {
    fn account(self) -> WriteCommandsAccount<'a>;
    fn account_admin(self) -> WriteCommandsAccountAdmin<'a>;
}

impl <'a> GetWriteCommandsAccount<'a> for WriteCommands<'a> {
    fn account(self) -> WriteCommandsAccount<'a> {
        WriteCommandsAccount::new(self)
    }

    fn account_admin(self) -> WriteCommandsAccountAdmin<'a> {
        WriteCommandsAccountAdmin::new(self)
    }
}
