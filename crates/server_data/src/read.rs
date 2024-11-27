use crate::db_manager::{InternalReading, ReadAccessProvider};

use self::common::ReadCommandsCommon;

pub mod common;

pub trait GetReadCommandsCommon<C> {
    fn common(self) -> ReadCommandsCommon<C>;
}

impl <C: ReadAccessProvider> GetReadCommandsCommon<C> for C {
    fn common(self) -> ReadCommandsCommon<C> {
        ReadCommandsCommon::new(self)
    }
}

pub trait DbReadCommon {
    async fn db_read<
        T: FnOnce(
                database::current::read::CurrentSyncReadCommands<
                    &mut database::DieselConnection,
                >,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, database::DieselDatabaseError>;
}

impl <I: InternalReading> DbReadCommon for I {
    async fn db_read<
        T: FnOnce(
                database::current::read::CurrentSyncReadCommands<
                    &mut database::DieselConnection,
                >,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, database::DieselDatabaseError> {
        self.db_read_raw(|c| cmd(database::current::read::CurrentSyncReadCommands::new(c))).await
    }
}
