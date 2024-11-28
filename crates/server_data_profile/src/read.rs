use profile::ReadCommandsProfile;
use profile_admin::ReadCommandsProfileAdmin;
use profile_admin_history::ReadCommandsProfileAdminHistory;
use server_data::db_manager::{InternalReading, ReadAccessProvider};

pub mod profile;
pub mod profile_admin;
pub mod profile_admin_history;

pub trait GetReadProfileCommands<'a> {
    fn profile(self) -> ReadCommandsProfile<'a>;
    fn profile_admin(self) -> ReadCommandsProfileAdmin<'a>;
    fn profile_admin_history(self) -> ReadCommandsProfileAdminHistory<'a>;
}

impl <'a, I: ReadAccessProvider<'a>> GetReadProfileCommands<'a> for I {
    fn profile(self) -> ReadCommandsProfile<'a> {
        ReadCommandsProfile::new(self.handle())
    }

    fn profile_admin(self) -> ReadCommandsProfileAdmin<'a> {
        ReadCommandsProfileAdmin::new(self.handle())
    }

    fn profile_admin_history(self) -> ReadCommandsProfileAdminHistory<'a> {
        ReadCommandsProfileAdminHistory::new(self.handle())
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
