use server_data::db_manager::{InternalReading, ReadAccessProvider};

use self::{account::ReadCommandsAccount, account_admin::ReadCommandsAccountAdmin};

pub mod account;
pub mod account_admin;

pub trait GetReadCommandsAccount<'a> {
    fn account(self) -> ReadCommandsAccount<'a>;
    fn account_admin(self) -> ReadCommandsAccountAdmin<'a>;
}

impl <'a, T: ReadAccessProvider<'a>> GetReadCommandsAccount<'a> for T {
    fn account(self) -> ReadCommandsAccount<'a> {
        ReadCommandsAccount::new(self.handle())
    }

    fn account_admin(self) -> ReadCommandsAccountAdmin<'a> {
        ReadCommandsAccountAdmin::new(self.handle())
    }
}

pub trait DbReadAccount {
    async fn db_read<
        T: FnOnce(
                database_account::current::read::CurrentSyncReadCommands<
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

impl <I: InternalReading> DbReadAccount for I {
    async fn db_read<
        T: FnOnce(
                database_account::current::read::CurrentSyncReadCommands<
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
        self.db_read_raw(|conn| {
            cmd(database_account::current::read::CurrentSyncReadCommands::new(conn))
        })
        .await
    }
}
