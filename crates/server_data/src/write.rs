//! Synchronous write commands combining cache and database operations.

use common::WriteCommandsCommon;

use crate::db_manager::{InternalWriting, WriteAccessProvider};

pub mod common;

/// One Account can do only one write command at a time.
pub struct AccountWriteLock;

/// Macro for writing to current database with transaction.
/// Calls await automatically.
///
/// ```ignore
/// use server::DataError;
/// use server::data::write::{define_write_commands, db_transaction};
///
/// define_write_commands!(WriteCommandsTest);
///
/// impl WriteCommandsTest<'_> {
///     pub async fn test(
///         &self,
///     ) -> server::result::Result<(), DataError> {
///         db_transaction!(self, move |mut cmds| {
///             Ok(())
///         })?;
///         Ok(())
///     }
/// }
/// ```
macro_rules! db_transaction {
    ($state:expr, move |mut $cmds:ident| $commands:expr) => {{
        $crate::IntoDataError::into_error($state.db_transaction_common(move |mut $cmds| ($commands)).await)
    }};
    ($state:expr, move |$cmds:ident| $commands:expr) => {{
        $crate::data::IntoDataError::into_error(
            $state.db_transaction_common(move |$cmds| ($commands)).await,
        )
    }};
}

// Make db_transaction available in all modules
pub(crate) use db_transaction;

pub trait GetWriteCommandsCommon<C> {
    fn common(self) -> WriteCommandsCommon<C>;
}

impl <I: WriteAccessProvider> GetWriteCommandsCommon<I> for I {
    fn common(self) -> WriteCommandsCommon<I> {
        WriteCommandsCommon::new(self)
    }
}

pub trait DbTransactionCommon {
    async fn db_transaction_common<
        T: FnOnce(
                database::current::write::CurrentSyncWriteCommands<
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

impl <I: InternalWriting> DbTransactionCommon for I {
    async fn db_transaction_common<
        T: FnOnce(
                database::current::write::CurrentSyncWriteCommands<
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
        self.db_transaction_raw(|c| cmd(database::current::write::CurrentSyncWriteCommands::new(c))).await
    }
}
