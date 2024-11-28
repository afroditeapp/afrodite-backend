//! Synchronous write commands combining cache and database operations.

use profile::WriteCommandsProfile;
use profile_admin::WriteCommandsProfileAdmin;
use profile_admin_history::WriteCommandsProfileAdminHistory;
use server_data::db_manager::{InternalWriting, WriteAccessProvider};

pub mod profile;
pub mod profile_admin;
pub mod profile_admin_history;

pub trait GetWriteCommandsProfile<'a> {
    fn profile(self) -> WriteCommandsProfile<'a>;
    fn profile_admin(self) -> WriteCommandsProfileAdmin<'a>;
    fn profile_admin_history(self) -> WriteCommandsProfileAdminHistory<'a>;
}

impl <'a, I: WriteAccessProvider<'a>> GetWriteCommandsProfile<'a> for I {
    fn profile(self) -> WriteCommandsProfile<'a> {
        WriteCommandsProfile::new(self.handle())
    }

    fn profile_admin(self) -> WriteCommandsProfileAdmin<'a> {
        WriteCommandsProfileAdmin::new(self.handle())
    }

    fn profile_admin_history(self) -> WriteCommandsProfileAdminHistory<'a> {
        WriteCommandsProfileAdminHistory::new(self.handle())
    }
}

pub trait DbTransactionProfile {
    async fn db_transaction<
        T: FnOnce(
                database_profile::current::write::CurrentSyncWriteCommands<
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

impl <I: InternalWriting> DbTransactionProfile for I {
    async fn db_transaction<
        T: FnOnce(
                database_profile::current::write::CurrentSyncWriteCommands<
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
        self.db_transaction_raw(|conn| cmd(database_profile::current::write::CurrentSyncWriteCommands::new(conn))).await
    }
}

pub trait DbTransactionProfileHistory {
    async fn db_transaction_history<
        T: FnOnce(
                database_profile::history::write::HistorySyncWriteCommands<
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

impl <I: InternalWriting> DbTransactionProfileHistory for I {
    async fn db_transaction_history<
        T: FnOnce(
                database_profile::history::write::HistorySyncWriteCommands<
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
        self.db_transaction_history_raw(|conn| cmd(database_profile::history::write::HistorySyncWriteCommands::new(conn))).await
    }
}
