//! Synchronous write commands combining cache and database operations.

use account::WriteCommandsAccount;
use account_admin::WriteCommandsAccountAdmin;
use server_data::db_manager::{InternalWriting, WriteAccessProvider};

pub mod account;
pub mod account_admin;

pub trait GetWriteCommandsAccount<C> {
    fn account(self) -> WriteCommandsAccount<C>;
    fn account_admin(self) -> WriteCommandsAccountAdmin<C>;
}

impl<C: WriteAccessProvider> GetWriteCommandsAccount<C> for C {
    fn account(self) -> WriteCommandsAccount<C> {
        WriteCommandsAccount::new(self)
    }

    fn account_admin(self) -> WriteCommandsAccountAdmin<C> {
        WriteCommandsAccountAdmin::new(self)
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
