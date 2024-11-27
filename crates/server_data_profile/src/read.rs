use profile::ReadCommandsProfile;
use profile_admin::ReadCommandsProfileAdmin;
use profile_admin_history::ReadCommandsProfileAdminHistory;
use server_data::db_manager::{InternalReading, ReadAccessProvider};

pub mod profile;
pub mod profile_admin;
pub mod profile_admin_history;

pub trait GetReadProfileCommands<C> {
    fn profile(self) -> ReadCommandsProfile<C>;
    fn profile_admin(self) -> ReadCommandsProfileAdmin<C>;
    fn profile_admin_history(self) -> ReadCommandsProfileAdminHistory<C>;
}

impl <I: ReadAccessProvider> GetReadProfileCommands<I> for I {
    fn profile(self) -> ReadCommandsProfile<I> {
        ReadCommandsProfile::new(self)
    }

    fn profile_admin(self) -> ReadCommandsProfileAdmin<I> {
        ReadCommandsProfileAdmin::new(self)
    }

    fn profile_admin_history(self) -> ReadCommandsProfileAdminHistory<I> {
        ReadCommandsProfileAdminHistory::new(self)
    }
}

pub trait DbReadProfile {
    async fn db_read<
        T: FnOnce(
                database_profile::current::read::CurrentSyncReadCommands<
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

impl <I: InternalReading> DbReadProfile for I {
    async fn db_read<
        T: FnOnce(
                database_profile::current::read::CurrentSyncReadCommands<
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
            cmd(database_profile::current::read::CurrentSyncReadCommands::new(conn))
        })
        .await
    }
}

pub trait DbReadProfileHistory {
    async fn db_read_history<
        T: FnOnce(
                database_profile::history::read::HistorySyncReadCommands<
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

impl <I: InternalReading> DbReadProfileHistory for I {
    async fn db_read_history<
        T: FnOnce(
                database_profile::history::read::HistorySyncReadCommands<
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
        self.db_read_history_raw(|conn| {
            cmd(database_profile::history::read::HistorySyncReadCommands::new(conn))
        })
        .await
    }
}
