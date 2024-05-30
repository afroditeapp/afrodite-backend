use std::future::Future;

use model::AccountId;
pub use server_common::app::*;

use crate::{
    db_manager::SyncWriteHandleRef, event::EventManagerWithCacheReference, read::{ReadCommands, ReadCommandsContainer}, write_commands::WriteCmds, write_concurrent::{ConcurrentWriteAction, ConcurrentWriteSelectorHandle}, DataError
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

    // fn write<
    //     CmdResult: Send + 'static,
    //     Cmd: Future<Output = crate::result::Result<CmdResult, DataError>> + Send,
    //     GetCmd,
    // >(
    //     &self,
    //     write_cmd: GetCmd,
    // ) -> impl std::future::Future<Output = crate::result::Result<CmdResult, DataError>> + Send where GetCmd: FnOnce(SyncWriteHandleRef<'_>) -> Cmd + Send + 'static;

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

pub trait ReadData {
    fn read(&self) -> ReadCommandsContainer;
}

pub trait EventManagerProvider {
    fn event_manager(&self) -> EventManagerWithCacheReference<'_>;
}
