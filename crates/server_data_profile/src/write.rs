//! Synchronous write commands combining cache and database operations.

use profile::WriteCommandsProfile;
use profile_admin::WriteCommandsProfileAdmin;
use profile_admin_history::WriteCommandsProfileAdminHistory;
use server_data::db_manager::{InternalWriting, WriteAccessProvider};

pub mod profile;
pub mod profile_admin;
pub mod profile_admin_history;

pub trait GetWriteCommandsProfile: Sized {
    fn profile(self) -> WriteCommandsProfile<Self>;
    fn profile_admin(self) -> WriteCommandsProfileAdmin<Self>;
    fn profile_admin_history(self) -> WriteCommandsProfileAdminHistory<Self>;
}

impl <I: WriteAccessProvider> GetWriteCommandsProfile for I {
    fn profile(self) -> WriteCommandsProfile<Self> {
        WriteCommandsProfile::new(self)
    }

    fn profile_admin(self) -> WriteCommandsProfileAdmin<Self> {
        WriteCommandsProfileAdmin::new(self)
    }

    fn profile_admin_history(self) -> WriteCommandsProfileAdminHistory<Self> {
        WriteCommandsProfileAdminHistory::new(self)
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
