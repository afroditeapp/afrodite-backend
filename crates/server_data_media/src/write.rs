//! Synchronous write commands combining cache and database operations.

use media::WriteCommandsMedia;
use media_admin::WriteCommandsMediaAdmin;
use server_data::db_manager::{InternalWriting, WriteAccessProvider};

pub mod media;
pub mod media_admin;

pub trait GetWriteCommandsMedia<'a> {
    fn media(self) -> WriteCommandsMedia<'a>;
    fn media_admin(self) -> WriteCommandsMediaAdmin<'a>;
}

impl <'a, I: WriteAccessProvider<'a>> GetWriteCommandsMedia<'a> for I {
    fn media(self) -> WriteCommandsMedia<'a> {
        WriteCommandsMedia::new(self.handle())
    }

    fn media_admin(self) -> WriteCommandsMediaAdmin<'a> {
        WriteCommandsMediaAdmin::new(self.handle())
    }
}

pub trait DbTransactionMedia {
    async fn db_transaction<
        T: FnOnce(
                database_media::current::write::CurrentSyncWriteCommands<
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

impl <I: InternalWriting> DbTransactionMedia for I {
    async fn db_transaction<
        T: FnOnce(
                database_media::current::write::CurrentSyncWriteCommands<
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
        self.db_transaction_raw(|conn| cmd(database_media::current::write::CurrentSyncWriteCommands::new(conn))).await
    }
}
