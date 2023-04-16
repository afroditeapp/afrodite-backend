//! Database writing commands
//!

use std::{sync::Arc, future::Future};

use api_client::models::AccountId;
use axum::{extract::BodyStream, RequestPartsExt};
use error_stack::{Result, ResultExt, Report};
use serde::Serialize;
use tokio::sync::{MutexGuard, Mutex, mpsc, oneshot};
use tokio_stream::StreamExt;
use tracing::instrument::WithSubscriber;

use crate::{
    api::{model::{Account, AccountIdInternal, AccountSetup, ApiKey, Profile, AccountIdLight, ContentId, NewModerationRequest}, media::data::Moderation},
    config::Config,
    server::database::{sqlite::SqliteWriteHandle, DatabaseError, write::WriteCommands},
    utils::{ErrorConversion, IntoReportExt, AppendErrorTo},
};

use super::{
    current::write::CurrentDataWriteCommands,
    history::write::HistoryWriteCommands,
    sqlite::{HistoryUpdateJson, SqliteUpdateJson, CurrentDataWriteHandle, HistoryWriteHandle},
    utils::GetReadWriteCmd, cache::{DatabaseCache, WriteCacheJson}, file::{utils::{ FileDir}, file::ImageSlot}, RouterDatabaseWriteHandle,
};


use tokio::sync::oneshot::{Sender, Receiver};

pub type ResultSender<T> = oneshot::Sender<Result<T, DatabaseError>>;

/// Synchronized write commands.
#[derive(Debug)]
pub enum WriteCommand {
    Register{ s: ResultSender<AccountIdInternal>, account_id: AccountIdLight },
    SetNewApiKey{ s: ResultSender<()>, account_id: AccountIdInternal, key: ApiKey },
    UpdateAccount{ s: ResultSender<()>, account_id: AccountIdInternal, account: Account },
    UpdateAccountSetup{ s: ResultSender<()>, account_id: AccountIdInternal, account_setup: AccountSetup },
    UpdateProfile{ s: ResultSender<()>, account_id: AccountIdInternal, profile: Profile },
    SetModerationRequest{ s: ResultSender<()>, account_id: AccountIdInternal, request: NewModerationRequest },
    GetModerationListAndCreateNewIfNecessary { s: ResultSender<Vec<Moderation>>, account_id: AccountIdInternal },
}


/// Concurrent write commands.
#[derive(Debug)]
pub enum ConcurrentWriteCommand {
    SaveToSlot { result_sender: ResultSender<ContentId>, account_id: AccountIdInternal },
}

#[derive(Debug)]
pub struct WriteCommandRunnerQuitHandle {
    handle: tokio::task::JoinHandle<()>,
}

impl WriteCommandRunnerQuitHandle {
    pub async fn quit(self) -> Result<(), DatabaseError> {
        self.handle.await.into_error(DatabaseError::CommandRunnerQuit)
    }
}

#[derive(Debug, Clone)]
pub struct WriteCommandRunnerHandle {
    sender: mpsc::Sender<WriteCommand>,
}

impl WriteCommandRunnerHandle {
    pub async fn register(&self, account_id: AccountIdLight) -> Result<AccountIdInternal, DatabaseError> {
        self.send_event(|s| WriteCommand::Register { s, account_id }).await
    }

    pub async fn set_new_api_key(&self, account_id: AccountIdInternal, key: ApiKey) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::SetNewApiKey { s, account_id, key }).await
    }

    pub async fn update_account(&self, account_id: AccountIdInternal, account: Account) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::UpdateAccount { s, account_id, account }).await
    }

    pub async fn update_account_setup(&self, account_id: AccountIdInternal, account_setup: AccountSetup) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::UpdateAccountSetup { s, account_id, account_setup }).await
    }

    pub async fn update_profile(&self, account_id: AccountIdInternal, profile: Profile) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::UpdateProfile { s, account_id, profile }).await
    }

    pub async fn set_moderation_request(&self, account_id: AccountIdInternal, request: NewModerationRequest) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::SetModerationRequest { s, account_id, request }).await
    }

    pub async fn get_moderation_list_and_create_if_necessary(&self, account_id: AccountIdInternal) -> Result<Vec<Moderation>, DatabaseError> {
        self.send_event(|s| WriteCommand::GetModerationListAndCreateNewIfNecessary { s, account_id }).await
    }

    async fn send_event<T>(&self, get_event: impl FnOnce(ResultSender<T>) -> WriteCommand) -> Result<T, DatabaseError> {
        let (result_sender, receiver) = oneshot::channel();
        self.sender.send(get_event(result_sender)).await.into_error(DatabaseError::CommandSendingFailed)?;
        receiver.await.into_error(DatabaseError::CommandResultReceivingFailed)?
    }
}

pub struct WriteCommandRunner {
    receiver: mpsc::Receiver<WriteCommand>,
    write_handle: RouterDatabaseWriteHandle,
    config: Arc<Config>,
}


impl WriteCommandRunner {
    pub fn new_channel() -> (WriteCommandRunnerHandle, mpsc::Receiver<WriteCommand>) {
        let (sender, receiver) = mpsc::channel(1);

        let runner_handle = WriteCommandRunnerHandle {
            sender,
        };
        (runner_handle, receiver)
    }


    pub fn new(write_handle: RouterDatabaseWriteHandle, receiver: mpsc::Receiver<WriteCommand>, config: Arc<Config>) -> WriteCommandRunnerQuitHandle {
        let runner = Self {
            receiver,
            write_handle,
            config,
        };

        let handle = tokio::spawn(runner.run());

        let quit_handle = WriteCommandRunnerQuitHandle {
            handle,
        };

        quit_handle
    }

    /// Runs until web server part of the server quits.
    pub async fn run(mut self) {
        loop {
            match self.receiver.recv().await {
                Some(cmd) => self.handle_cmd(cmd).await,
                None => {
                    tracing::info!("Write command runner closed");
                    break;
                },
            }
        }
    }

    pub async fn handle_cmd(&self, cmd: WriteCommand) {
        match cmd {
            WriteCommand::SetNewApiKey { s, account_id, key } =>
                self.write().set_new_api_key(account_id, key).await.send(s),
            WriteCommand::Register { s, account_id } =>
                self.write_handle.register(account_id, &self.config).await.send(s),
            WriteCommand::UpdateAccount { s, account_id, account } =>
                self.write().update_json(account_id, &account).await.send(s),
            WriteCommand::UpdateAccountSetup { s, account_id, account_setup } =>
                self.write().update_json(account_id, &account_setup).await.send(s),
            WriteCommand::UpdateProfile { s, account_id, profile } =>
                self.write().update_json(account_id, &profile).await.send(s),
            WriteCommand::SetModerationRequest { s, account_id, request } =>
                self.write().set_moderation_request(account_id, request).await.send(s),
            WriteCommand::GetModerationListAndCreateNewIfNecessary { s, account_id } =>
                self.write().moderation_get_list_and_create_new_if_necessary(account_id).await.send(s),
        }
    }

    fn write(&self) -> WriteCommands {
        self.write_handle.user_write_commands()
    }
}

trait SendBack<T>: Sized {
    fn send(self, s: ResultSender<T>);
}

impl <D> SendBack<D> for Result<D, DatabaseError> {
    fn send(self, s: ResultSender<D>) {
        match s.send(self) {
            Ok(()) => (),
            Err(_) => {
                // Most likely request handler was dropped as client closed the
                // connection.
                ()
            }
        }
    }
}
