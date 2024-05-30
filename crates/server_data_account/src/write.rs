//! Synchronous write commands combining cache and database operations.

use account::WriteCommandsAccount;
use account_admin::WriteCommandsAccountAdmin;
use server_data::{write::{WriteCommandsProvider}};

pub mod account;
pub mod account_admin;

pub trait GetWriteCommandsAccount<C: WriteCommandsProvider> {
    fn account(self) -> WriteCommandsAccount<C>;
    fn account_admin(self) -> WriteCommandsAccountAdmin<C>;
}

impl <C: WriteCommandsProvider> GetWriteCommandsAccount<C> for C {
    fn account(self) -> WriteCommandsAccount<C> {
        WriteCommandsAccount::new(self)
    }

    fn account_admin(self) -> WriteCommandsAccountAdmin<C> {
        WriteCommandsAccountAdmin::new(self)
    }
}
