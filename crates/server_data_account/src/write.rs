//! Synchronous write commands combining cache and database operations.

use account::WriteCommandsAccount;
use account_admin::WriteCommandsAccountAdmin;
use server_data::db_manager::{InternalWriting, WriteAccessProvider};

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

pub trait DbTransactionAccount {
    async fn db_transaction<
        T: FnOnce(
                database_account::current::write::CurrentSyncWriteCommands<
                    &mut server_data::DieselConnection,
                >,
            ) -> error_stack::Result<R, server_data::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, server_data::DieselDatabaseError>;
}


impl <I: InternalWriting> DbTransactionAccount for I {
    async fn db_transaction<
        T: FnOnce(
                database_account::current::write::CurrentSyncWriteCommands<
                    &mut server_data::DieselConnection,
                >,
            ) -> error_stack::Result<R, server_data::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, server_data::DieselDatabaseError> {
        self.db_transaction_raw(|conn| cmd(database_account::current::write::CurrentSyncWriteCommands::new(conn))).await
    }
}
