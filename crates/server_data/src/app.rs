use std::{future::Future, sync::Arc};

use config::Config;
use model::AccountId;
pub use server_common::app::*;

use crate::{
    db_manager::{InternalWriting, RouterDatabaseReadHandle}, event::EventManagerWithCacheReference, write_commands::WriteCmds, write_concurrent::{ConcurrentWriteAction, ConcurrentWriteProfileHandleBlocking, ConcurrentWriteSelectorHandle}, DataError
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

    fn concurrent_write_profile_blocking<
        CmdResult: Send + 'static,
        WriteCmd: FnOnce(ConcurrentWriteProfileHandleBlocking) -> CmdResult + Send + 'static,
    >(
        &self,
        account: AccountId,
        write_cmd: WriteCmd,
    ) -> impl std::future::Future<Output = crate::result::Result<CmdResult, DataError>> + Send;
}

pub trait ReadData {
    fn read(&self) -> &RouterDatabaseReadHandle;
}

pub trait EventManagerProvider {
    fn event_manager(&self) -> EventManagerWithCacheReference<'_>;
}

impl <I: InternalWriting> EventManagerProvider for I {
    fn event_manager(&self) -> EventManagerWithCacheReference<'_> {
        EventManagerWithCacheReference::new(self.cache(), self.push_notification_sender())
    }
}


pub trait GetConfig {
    fn config(&self) -> &Config;
    fn config_arc(&self) -> Arc<Config>;
}

impl <I: InternalWriting> GetConfig for I {
    fn config(&self) -> &config::Config {
        InternalWriting::config(self)
    }

    fn config_arc(&self) -> std::sync::Arc<config::Config> {
        InternalWriting::config_arc(self)
    }
}

pub trait GetEmailSender {
    fn email_sender(&self) -> &EmailSenderImpl;
}

impl <I: InternalWriting> GetEmailSender for I {
    fn email_sender(&self) -> &EmailSenderImpl {
        InternalWriting::email_sender(self)
    }
}
