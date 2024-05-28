
use std::future::Future;

use model::AccountId;

use crate::{event::EventManagerWithCacheReference, read::ReadCommands, write_commands::WriteCmds, write_concurrent::{ConcurrentWriteAction, ConcurrentWriteSelectorHandle}, DataError};

pub use server_common::app::*;

#[async_trait::async_trait]
pub trait WriteData {
    async fn write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = crate::result::Result<CmdResult, DataError>> + Send + 'static,
        GetCmd: FnOnce(WriteCmds) -> Cmd + Send + 'static,
    >(
        &self,
        cmd: GetCmd,
    ) -> crate::result::Result<CmdResult, DataError>;

    async fn write_concurrent<
        CmdResult: Send + 'static,
        Cmd: Future<Output = ConcurrentWriteAction<CmdResult>> + Send + 'static,
        GetCmd: FnOnce(ConcurrentWriteSelectorHandle) -> Cmd + Send + 'static,
    >(
        &self,
        account: AccountId,
        cmd: GetCmd,
    ) -> crate::result::Result<CmdResult, DataError>;
}

pub trait ReadData {
    fn read(&self) -> ReadCommands<'_>;
}

pub trait EventManagerProvider {
    fn event_manager(&self) -> EventManagerWithCacheReference<'_>;
}
