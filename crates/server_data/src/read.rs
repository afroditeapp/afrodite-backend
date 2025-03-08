use common_admin::ReadCommandsCommonAdmin;
use common_history::ReadCommandsCommonHistory;

use self::common::ReadCommandsCommon;
use crate::db_manager::{InternalReading, ReadAccessProvider};

pub mod common;
mod common_admin;
mod common_history;

pub trait GetReadCommandsCommon<'a> {
    fn common(self) -> ReadCommandsCommon<'a>;
    fn common_admin(self) -> ReadCommandsCommonAdmin<'a>;
    fn common_history(self) -> ReadCommandsCommonHistory<'a>;
}

impl<'a, C: ReadAccessProvider<'a>> GetReadCommandsCommon<'a> for C {
    fn common(self) -> ReadCommandsCommon<'a> {
        ReadCommandsCommon::new(self.handle())
    }
    fn common_admin(self) -> ReadCommandsCommonAdmin<'a> {
        ReadCommandsCommonAdmin::new(self.handle())
    }
    fn common_history(self) -> ReadCommandsCommonHistory<'a> {
        ReadCommandsCommonHistory::new(self.handle())
    }
}

pub trait DbRead {
    async fn db_read<
        T: FnOnce(
                database::DbReadMode<'_>,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, database::DieselDatabaseError>;

    async fn db_read_history<
        T: FnOnce(
                database::DbReadModeHistory<'_>,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, database::DieselDatabaseError>;
}

impl<I: InternalReading> DbRead for I {
    async fn db_read<
        T: FnOnce(
                database::DbReadMode<'_>,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, database::DieselDatabaseError> {
        self.db_read_raw(cmd).await
    }

    async fn db_read_history<
        T: FnOnce(
                database::DbReadModeHistory<'_>,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, database::DieselDatabaseError> {
        self.db_read_history_raw(cmd).await
    }
}
