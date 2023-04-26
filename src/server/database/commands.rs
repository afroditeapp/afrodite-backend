//! Database writing commands
//!

use std::{collections::HashSet, future::Future, sync::Arc};


use axum::{extract::BodyStream};
use error_stack::{Result};


use tokio::{
    sync::{
        mpsc, oneshot, OwnedSemaphorePermit, RwLock, Semaphore,
    },
    task::JoinHandle,
};
use tokio_stream::StreamExt;


use crate::{
    api::{
        media::data::{Moderation, HandleModerationRequest},
        model::{
            Account, AccountIdInternal, AccountIdLight, AccountSetup, ApiKey, ContentId,
            NewModerationRequest, Profile,
        },
    },
    config::Config,
    server::database::{write::WriteCommands, DatabaseError},
    utils::{ErrorConversion, IntoReportExt},
};

use super::{
    file::{file::ImageSlot},
    sqlite::{SqliteUpdateJson},
    RouterDatabaseWriteHandle,
};

const CONCURRENT_WRITE_COMMAND_LIMIT: usize = 10;



pub type ResultSender<T> = oneshot::Sender<Result<T, DatabaseError>>;

/// Synchronized write commands.
#[derive(Debug)]
pub enum WriteCommand {
    Register {
        s: ResultSender<AccountIdInternal>,
        account_id: AccountIdLight,
    },
    SetNewApiKey {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        key: ApiKey,
    },
    UpdateAccount {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        account: Account,
    },
    UpdateAccountSetup {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        account_setup: AccountSetup,
    },
    UpdateProfile {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        profile: Profile,
    },
    SetModerationRequest {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        request: NewModerationRequest,
    },
    GetModerationListAndCreateNewIfNecessary {
        s: ResultSender<Vec<Moderation>>,
        account_id: AccountIdInternal,
    },
    SaveToSlot {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    },
    UpdateModeration {
        s: ResultSender<()>,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    },
}

/// Concurrent write commands.
#[derive(Debug)]
pub enum ConcurrentWriteCommand {
    SaveToTmp {
        s: ResultSender<ContentId>,
        account_id: AccountIdInternal,
        data_stream: BodyStream,
    },
}

#[derive(Debug)]
pub struct WriteCommandRunnerQuitHandle {
    handle: tokio::task::JoinHandle<()>,
    handle_for_concurrent: tokio::task::JoinHandle<()>,
}

impl WriteCommandRunnerQuitHandle {
    pub async fn quit(self) -> Result<(), DatabaseError> {
        let e1 = self
            .handle
            .await
            .into_error(DatabaseError::CommandRunnerQuit);
        let e2 = self
            .handle_for_concurrent
            .await
            .into_error(DatabaseError::CommandRunnerQuit);

        match (e1, e2) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(e), Ok(())) | (Ok(()), Err(e)) => Err(e),
            (Err(mut e1), Err(e2)) => {
                e1.extend_one(e2);
                Err(e1)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct WriteCommandRunnerHandle {
    sender: mpsc::Sender<WriteCommand>,
    sender_for_concurrent: mpsc::Sender<ConcurrentMessage>,
}

impl WriteCommandRunnerHandle {
    pub async fn register(
        &self,
        account_id: AccountIdLight,
    ) -> Result<AccountIdInternal, DatabaseError> {
        self.send_event(|s| WriteCommand::Register { s, account_id })
            .await
    }

    pub async fn set_new_api_key(
        &self,
        account_id: AccountIdInternal,
        key: ApiKey,
    ) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::SetNewApiKey { s, account_id, key })
            .await
    }

    pub async fn update_account(
        &self,
        account_id: AccountIdInternal,
        account: Account,
    ) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::UpdateAccount {
            s,
            account_id,
            account,
        })
        .await
    }

    pub async fn update_account_setup(
        &self,
        account_id: AccountIdInternal,
        account_setup: AccountSetup,
    ) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::UpdateAccountSetup {
            s,
            account_id,
            account_setup,
        })
        .await
    }

    pub async fn update_profile(
        &self,
        account_id: AccountIdInternal,
        profile: Profile,
    ) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::UpdateProfile {
            s,
            account_id,
            profile,
        })
        .await
    }

    pub async fn set_moderation_request(
        &self,
        account_id: AccountIdInternal,
        request: NewModerationRequest,
    ) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::SetModerationRequest {
            s,
            account_id,
            request,
        })
        .await
    }

    pub async fn get_moderation_list_and_create_if_necessary(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, DatabaseError> {
        self.send_event(|s| WriteCommand::GetModerationListAndCreateNewIfNecessary {
            s,
            account_id,
        })
        .await
    }

    pub async fn update_moderation(
        &self,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    ) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::UpdateModeration {
            s,
            moderator_id,
            moderation_request_owner,
            result,
        })
        .await
    }

    pub async fn save_to_slot(
        &self,
        account_id: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::SaveToSlot {
            s,
            account_id,
            content_id,
            slot,
        })
        .await
    }

    pub async fn save_to_tmp(
        &self,
        account_id: AccountIdInternal,
        data_stream: BodyStream,
    ) -> Result<ContentId, DatabaseError> {
        self.send_event_to_concurrent_runner(|s| {
            (
                account_id.as_light(),
                ConcurrentWriteCommand::SaveToTmp {
                    s,
                    account_id,
                    data_stream,
                },
            )
        })
        .await
    }

    async fn send_event<T>(
        &self,
        get_event: impl FnOnce(ResultSender<T>) -> WriteCommand,
    ) -> Result<T, DatabaseError> {
        let (result_sender, receiver) = oneshot::channel();
        self.sender
            .send(get_event(result_sender))
            .await
            .into_error(DatabaseError::CommandSendingFailed)?;
        receiver
            .await
            .into_error(DatabaseError::CommandResultReceivingFailed)?
    }

    async fn send_event_to_concurrent_runner<T>(
        &self,
        get_event: impl FnOnce(ResultSender<T>) -> ConcurrentMessage,
    ) -> Result<T, DatabaseError> {
        let (result_sender, receiver) = oneshot::channel();
        self.sender_for_concurrent
            .send(get_event(result_sender))
            .await
            .into_error(DatabaseError::CommandSendingFailed)?;
        receiver
            .await
            .into_error(DatabaseError::CommandResultReceivingFailed)?
    }
}

pub struct WriteCommandRunner {
    receiver: mpsc::Receiver<WriteCommand>,
    write_handle: RouterDatabaseWriteHandle,
    config: Arc<Config>,
}

impl WriteCommandRunner {
    pub fn new_channel() -> (WriteCommandRunnerHandle, WriteCommandReceivers) {
        let (sender, receiver) = mpsc::channel(1);
        let (sender_for_concurrent, receiver_for_concurrent) = mpsc::channel(1);

        let runner_handle = WriteCommandRunnerHandle {
            sender,
            sender_for_concurrent,
        };
        (
            runner_handle,
            WriteCommandReceivers {
                receiver,
                receiver_for_concurrent,
            },
        )
    }

    pub fn new(
        write_handle: RouterDatabaseWriteHandle,
        receiver: WriteCommandReceivers,
        config: Arc<Config>,
    ) -> WriteCommandRunnerQuitHandle {
        let runner = Self {
            receiver: receiver.receiver,
            write_handle: write_handle.clone(),
            config: config.clone(),
        };

        let runner_for_concurrent = ConcurrentWriteCommandRunner::new(
            receiver.receiver_for_concurrent,
            write_handle,
            config,
        );

        let handle = tokio::spawn(runner.run());
        let handle_for_concurrent = tokio::spawn(runner_for_concurrent.run());

        let quit_handle = WriteCommandRunnerQuitHandle {
            handle,
            handle_for_concurrent,
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
                }
            }
        }
    }

    pub async fn handle_cmd(&self, cmd: WriteCommand) {
        match cmd {
            WriteCommand::SetNewApiKey { s, account_id, key } => {
                self.write().set_new_api_key(account_id, key).await.send(s)
            }
            WriteCommand::Register { s, account_id } => self
                .write_handle
                .register(account_id, &self.config)
                .await
                .send(s),
            WriteCommand::UpdateAccount {
                s,
                account_id,
                account,
            } => self.write().update_json(account_id, &account).await.send(s),
            WriteCommand::UpdateAccountSetup {
                s,
                account_id,
                account_setup,
            } => self
                .write()
                .update_json(account_id, &account_setup)
                .await
                .send(s),
            WriteCommand::UpdateProfile {
                s,
                account_id,
                profile,
            } => self.write().update_json(account_id, &profile).await.send(s),
            WriteCommand::SetModerationRequest {
                s,
                account_id,
                request,
            } => self
                .write()
                .set_moderation_request(account_id, request)
                .await
                .send(s),
            WriteCommand::GetModerationListAndCreateNewIfNecessary { s, account_id } => self
                .write()
                .moderation_get_list_and_create_new_if_necessary(account_id)
                .await
                .send(s),
            WriteCommand::SaveToSlot {
                s,
                account_id,
                content_id,
                slot,
            } => self
                .write()
                .save_to_slot(account_id, content_id, slot)
                .await
                .send(s),
            WriteCommand::UpdateModeration {
                s,
                moderator_id,
                moderation_request_owner,
                result
            } => self
                .write()
                .update_moderation(moderator_id, moderation_request_owner, result)
                .await
                .send(s)
        }
    }

    fn write(&self) -> WriteCommands {
        self.write_handle.user_write_commands()
    }
}

trait SendBack<T>: Sized {
    fn send(self, s: ResultSender<T>);
}

impl<D> SendBack<D> for Result<D, DatabaseError> {
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

type ConcurrentMessage = (AccountIdLight, ConcurrentWriteCommand);

pub struct WriteCommandReceivers {
    receiver: mpsc::Receiver<WriteCommand>,
    receiver_for_concurrent: mpsc::Receiver<ConcurrentMessage>,
}

pub struct ConcurrentWriteCommandRunner {
    receiver: mpsc::Receiver<ConcurrentMessage>,
    write_handle: RouterDatabaseWriteHandle,
    config: Arc<Config>,
    task_handles: Vec<JoinHandle<()>>,
}

#[derive(Default, Clone)]
pub struct AccountWriteLockManager {
    locks: Arc<RwLock<HashSet<AccountIdLight>>>,
}

#[must_use]
struct AccountWriteLockHandle {
    locks: Arc<RwLock<HashSet<AccountIdLight>>>,
    account: AccountIdLight,
}

impl AccountWriteLockManager {
    #[must_use]
    async fn set_as_running(&self, a: AccountIdLight) -> Option<AccountWriteLockHandle> {
        if self.locks.write().await.insert(a) {
            Some(AccountWriteLockHandle {
                locks: self.locks.clone(),
                account: a,
            })
        } else {
            None
        }
    }
}

impl AccountWriteLockHandle {
    async fn release(self) {
        self.locks.write().await.remove(&self.account);
    }
}

impl ConcurrentWriteCommandRunner {
    pub fn new(
        receiver: mpsc::Receiver<ConcurrentMessage>,
        write_handle: RouterDatabaseWriteHandle,
        config: Arc<Config>,
    ) -> Self {
        Self {
            receiver,
            write_handle,
            config,
            task_handles: vec![],
        }
    }

    /// Runs until web server part of the server quits.
    pub async fn run(mut self) {
        let task_limiter = Arc::new(Semaphore::new(CONCURRENT_WRITE_COMMAND_LIMIT));
        let mut skip = false;
        let cmd_owners = AccountWriteLockManager::default();
        loop {
            match self.receiver.recv().await {
                Some(_) if skip => (),
                Some((cmd_owner, cmd)) => {
                    let lock = match cmd_owners.set_as_running(cmd_owner).await {
                        None => {
                            // Cmd already running. Client handles that this is
                            // not possible.
                            continue;
                        }
                        Some(l) => l,
                    };

                    let permit = task_limiter.clone().acquire_owned().await;
                    match permit {
                        Ok(permit) => {
                            self.handle_cmd(cmd, permit, lock).await;
                        }
                        Err(e) => {
                            tracing::error!(
                                "Task limiter was closed. Skipping all next commands. Error: {}",
                                e
                            );
                            skip = true;
                            lock.release().await;
                        }
                    }
                }
                None => {
                    tracing::info!("Concurrent write command runner closed");
                    break;
                }
            }
        }

        for handle in self.task_handles {
            match handle.await {
                Ok(()) => (),
                Err(e) => {
                    tracing::error!("Concurrent task join failed: {}", e);
                }
            }
        }
    }

    async fn handle_cmd(
        &mut self,
        cmd: ConcurrentWriteCommand,
        p: OwnedSemaphorePermit,
        l: AccountWriteLockHandle,
    ) {
        match cmd {
            ConcurrentWriteCommand::SaveToTmp {
                s,
                account_id,
                data_stream,
            } => {
                self.start_cmd_task(p, l, s, move |w| async move {
                    w.user_write_commands_account()
                        .save_to_tmp(account_id, data_stream)
                        .await
                })
                .await;
            }
        }
    }

    async fn start_cmd_task<
        T: Send + 'static,
        F: Future<Output = Result<T, DatabaseError>> + Send + 'static,
    >(
        &mut self,
        permit: OwnedSemaphorePermit,
        l: AccountWriteLockHandle,
        s: ResultSender<T>,
        f: impl FnOnce(RouterDatabaseWriteHandle) -> F + Send + 'static,
    ) {
        let w = self.write_handle.clone();

        self.task_handles.push(tokio::spawn(async move {
            let r = f(w).await;
            l.release().await; // Make sure that next cmd is possible to make when response is returned to the clent.
            r.send(s);
            drop(permit);
        }));
    }

    fn write(&self) -> WriteCommands {
        self.write_handle.user_write_commands()
    }

    async fn handle_cmd_in_task(_cmd: ConcurrentWriteCommand) {}
}
