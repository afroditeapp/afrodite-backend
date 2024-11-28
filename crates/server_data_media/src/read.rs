use media::ReadCommandsMedia;
use media_admin::ReadCommandsMediaAdmin;
use server_data::db_manager::{InternalReading, ReadAccessProvider};

pub mod media;
pub mod media_admin;

pub trait GetReadMediaCommands<'a> {
    fn media(self) -> ReadCommandsMedia<'a>;
    fn media_admin(self) -> ReadCommandsMediaAdmin<'a>;
}

impl <'a, I: ReadAccessProvider<'a>> GetReadMediaCommands<'a> for I {
    fn media(self) -> ReadCommandsMedia<'a> {
        ReadCommandsMedia::new(self.handle())
    }

    fn media_admin(self) -> ReadCommandsMediaAdmin<'a> {
        ReadCommandsMediaAdmin::new(self.handle())
    }
}

pub trait DbReadMedia {
    async fn db_read<
        T: FnOnce(
                database_media::current::read::CurrentSyncReadCommands<
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

impl <I: InternalReading> DbReadMedia for I {
    async fn db_read<
        T: FnOnce(
                database_media::current::read::CurrentSyncReadCommands<
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
        self.db_read_raw(|conn| cmd(database_media::current::read::CurrentSyncReadCommands::new(conn))).await
    }
}
