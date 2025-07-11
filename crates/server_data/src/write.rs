//! Synchronous write commands combining cache and database operations.

use common::WriteCommandsCommon;
use common_admin::WriteCommandsCommonAdmin;

use crate::db_manager::{InternalWriting, WriteAccessProvider};

pub mod common;
pub mod common_admin;
pub mod common_history;

/// One Account can do only one write command at a time.
pub struct AccountWriteLock;

pub trait GetWriteCommandsCommon {
    fn common(&self) -> WriteCommandsCommon<'_>;
    fn common_admin(&self) -> WriteCommandsCommonAdmin<'_>;
    fn common_history(&self) -> common_history::WriteCommandsCommonHistory<'_>;
}

impl<I: WriteAccessProvider> GetWriteCommandsCommon for I {
    fn common(&self) -> WriteCommandsCommon<'_> {
        WriteCommandsCommon::new(self.handle())
    }

    fn common_admin(&self) -> WriteCommandsCommonAdmin<'_> {
        WriteCommandsCommonAdmin::new(self.handle())
    }

    fn common_history(&self) -> common_history::WriteCommandsCommonHistory<'_> {
        common_history::WriteCommandsCommonHistory::new(self.handle())
    }
}

pub trait DbTransaction {
    async fn db_transaction<
        T: FnOnce(
                database::DbWriteMode<'_>,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, database::DieselDatabaseError>;
}

impl<I: InternalWriting> DbTransaction for I {
    async fn db_transaction<
        T: FnOnce(
                database::DbWriteMode<'_>,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, database::DieselDatabaseError> {
        self.db_transaction_raw(cmd).await
    }
}

pub trait DbTransactionHistory {
    async fn db_transaction_history<
        T: FnOnce(
                database::DbWriteModeHistory<'_>,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, database::DieselDatabaseError>;
}

impl<I: InternalWriting> DbTransactionHistory for I {
    async fn db_transaction_history<
        T: FnOnce(
                database::DbWriteModeHistory<'_>,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, database::DieselDatabaseError> {
        self.db_transaction_history_raw(cmd).await
    }
}
