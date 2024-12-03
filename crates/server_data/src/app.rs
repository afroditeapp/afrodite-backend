use std::{future::Future, sync::Arc};

use axum::extract::ws::WebSocket;
use config::{file::EmailAddress, Config};
use futures::future::BoxFuture;
use model::{
    Account, AccountId, AccountIdInternal, PendingNotification, PendingNotificationWithData,
    SyncDataVersionFromClient,
};
use model_server_data::SignInWithInfo;
pub use server_common::app::*;
use server_common::websocket::WebSocketError;

use crate::{
    db_manager::{InternalWriting, RouterDatabaseReadHandle},
    event::EventManagerWithCacheReference,
    statistics::ProfileStatisticsCache,
    write_commands::{WriteCmds, WriteCommandRunnerHandle},
    write_concurrent::{
        ConcurrentWriteAction, ConcurrentWriteProfileHandleBlocking, ConcurrentWriteSelectorHandle,
    },
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

pub trait ProfileStatisticsCacheProvider {
    fn profile_statistics_cache(&self) -> &ProfileStatisticsCache;
}

pub trait EventManagerProvider {
    fn event_manager(&self) -> EventManagerWithCacheReference<'_>;
}

impl<I: InternalWriting> EventManagerProvider for I {
    fn event_manager(&self) -> EventManagerWithCacheReference<'_> {
        EventManagerWithCacheReference::new(self.cache(), self.push_notification_sender())
    }
}

pub trait GetConfig {
    fn config(&self) -> &Config;
    fn config_arc(&self) -> Arc<Config>;
}

impl<I: InternalWriting> GetConfig for I {
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

impl<I: InternalWriting> GetEmailSender for I {
    fn email_sender(&self) -> &EmailSenderImpl {
        InternalWriting::email_sender(self)
    }
}

/// Data commands which have cross component dependencies.
///
/// This exists to avoid recompiling most of the crates when data layer crate
/// is edited.
pub trait DataAllUtils: Send + Sync + 'static {
    fn update_unlimited_likes<'a>(
        &self,
        write_command_runner: &'a WriteCommandRunnerHandle,
        id: AccountIdInternal,
        unlimited_likes: bool,
    ) -> BoxFuture<'a, server_common::result::Result<(), DataError>>;

    fn register_impl<'a>(
        &self,
        write_command_runner: &'a WriteCommandRunnerHandle,
        sign_in_with: SignInWithInfo,
        email: Option<EmailAddress>,
    ) -> BoxFuture<'a, server_common::result::Result<AccountIdInternal, DataError>>;

    fn handle_new_websocket_connection<'a>(
        &self,
        config: &'a Config,
        read_handle: &'a RouterDatabaseReadHandle,
        write_handle: &'a WriteCommandRunnerHandle,
        socket: &'a mut WebSocket,
        id: AccountIdInternal,
        sync_versions: Vec<SyncDataVersionFromClient>,
    ) -> BoxFuture<'a, server_common::result::Result<(), WebSocketError>>;

    fn get_push_notification_data<'a>(
        &self,
        read_handle: &'a RouterDatabaseReadHandle,
        id: AccountIdInternal,
        notification_value: PendingNotification,
    ) -> BoxFuture<'a, PendingNotificationWithData>;

    fn complete_initial_setup<'a>(
        &self,
        config: &'a Config,
        read_handle: &'a RouterDatabaseReadHandle,
        write_handle: &'a WriteCommandRunnerHandle,
        id: AccountIdInternal,
    ) -> BoxFuture<'a, server_common::result::Result<Account, DataError>>;

    fn is_match<'a>(
        &self,
        read_handle: &'a RouterDatabaseReadHandle,
        account0: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> BoxFuture<'a, server_common::result::Result<bool, DataError>>;
}
