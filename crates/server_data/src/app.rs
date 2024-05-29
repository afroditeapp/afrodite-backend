use std::future::Future;

use model::AccountId;
pub use server_common::app::*;

use crate::{
    event::EventManagerWithCacheReference,
    read::ReadCommands,
    write_commands::WriteCmds,
    write_concurrent::{ConcurrentWriteAction, ConcurrentWriteSelectorHandle},
    DataError,
};

pub trait WriteData {
    fn write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = crate::result::Result<CmdResult, DataError>> + Send + 'static,
        GetCmd: FnOnce(WriteCmds) -> Cmd + Send + 'static,
    >(
        &self,
        cmd: GetCmd,
    ) -> impl std::future::Future<Output = crate::result::Result<CmdResult, DataError>> + Send;

    fn write_concurrent<
        CmdResult: Send + 'static,
        Cmd: Future<Output = ConcurrentWriteAction<CmdResult>> + Send + 'static,
        GetCmd: FnOnce(ConcurrentWriteSelectorHandle) -> Cmd + Send + 'static,
    >(
        &self,
        account: AccountId,
        cmd: GetCmd,
    ) -> impl std::future::Future<Output = crate::result::Result<CmdResult, DataError>> + Send;
}

pub trait ReadData<T> {
    fn read(&self) -> T;
}

pub trait EventManagerProvider {
    fn event_manager(&self) -> EventManagerWithCacheReference<'_>;
}
