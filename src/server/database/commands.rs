//! Database writing commands
//!

pub mod account;
pub mod media;
pub mod profile;
pub mod chat;

use std::{collections::HashSet, future::Future, net::SocketAddr, sync::Arc};

use axum::extract::BodyStream;
use error_stack::Result;

use tokio::{
    sync::{mpsc, oneshot, OwnedSemaphorePermit, RwLock, Semaphore},
    task::JoinHandle,
};
use tokio_stream::StreamExt;

use crate::{
    api::{
        media::data::{HandleModerationRequest, Moderation},
        model::{
            Account, AccountIdInternal, AccountIdLight, AccountSetup, AuthPair, ContentId,
            Location, ModerationRequestContent, ProfileLink,
            ProfileUpdateInternal, SignInWithInfo,
        },
    },
    config::Config,
    server::database::{write::WriteCommands, DatabaseError},
    utils::{ErrorConversion, IntoReportExt},
};

use self::{media::{MediaWriteCommandRunnerHandle, MediaWriteCommand}, profile::{ProfileWriteCommandRunnerHandle, ProfileWriteCommand}, account::{AccountWriteCommand, AccountWriteCommandRunnerHandle}, chat::{ChatWriteCommand, ChatWriteCommandRunnerHandle}};

use super::{file::file::ImageSlot, RouterDatabaseWriteHandle};

const CONCURRENT_WRITE_COMMAND_LIMIT: usize = 10;

pub type ResultSender<T> = oneshot::Sender<Result<T, DatabaseError>>;

/// Synchronized write commands.
#[derive(Debug)]
pub enum WriteCommand {
    SetNewAuthPair {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        pair: AuthPair,
        address: Option<SocketAddr>,
    },
    Logout {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
    },
    EndConnectionSession {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
    },
    Account(AccountWriteCommand),
    Profile(ProfileWriteCommand),
    Media(MediaWriteCommand),
    Chat(ChatWriteCommand),
}

impl From<AccountWriteCommand> for WriteCommand {
    fn from(value: AccountWriteCommand) -> Self {
        Self::Account(value)
    }
}

impl From<ProfileWriteCommand> for WriteCommand {
    fn from(value: ProfileWriteCommand) -> Self {
        Self::Profile(value)
    }
}

impl From<MediaWriteCommand> for WriteCommand {
    fn from(value: MediaWriteCommand) -> Self {
        Self::Media(value)
    }
}

impl From<ChatWriteCommand> for WriteCommand {
    fn from(value: ChatWriteCommand) -> Self {
        Self::Chat(value)
    }
}


/// Concurrent write commands.
#[derive(Debug)]
pub enum ConcurrentWriteCommand {
    SaveToTmp {
        s: ResultSender<ContentId>,
        account_id: AccountIdInternal,
        data_stream: BodyStream,
    },
    ResetProfileIterator {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
    },
    NextProfiles {
        s: ResultSender<Vec<ProfileLink>>,
        account_id: AccountIdInternal,
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
    pub fn account(
        &self,
    ) -> AccountWriteCommandRunnerHandle {
        AccountWriteCommandRunnerHandle { handle: self }
    }

    pub fn media(
        &self,
    ) -> MediaWriteCommandRunnerHandle {
        MediaWriteCommandRunnerHandle { handle: self }
    }

    pub fn profile(
        &self,
    ) -> ProfileWriteCommandRunnerHandle {
        ProfileWriteCommandRunnerHandle { handle: self }
    }

    pub fn chat(
        &self,
    ) -> ChatWriteCommandRunnerHandle {
        ChatWriteCommandRunnerHandle { handle: self }
    }

    pub async fn set_new_auth_pair(
        &self,
        account_id: AccountIdInternal,
        pair: AuthPair,
        address: Option<SocketAddr>,
    ) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::SetNewAuthPair {
            s,
            account_id,
            pair,
            address,
        })
        .await
    }

    pub async fn logout(&self, account_id: AccountIdInternal) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::Logout { s, account_id })
            .await
    }

    pub async fn end_connection_session(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<(), DatabaseError> {
        self.send_event(|s| WriteCommand::EndConnectionSession { s, account_id })
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

    pub async fn next_profiles(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<ProfileLink>, DatabaseError> {
        self.send_event_to_concurrent_runner(|s| {
            (
                account_id.as_light(),
                ConcurrentWriteCommand::NextProfiles { s, account_id },
            )
        })
        .await
    }

    pub async fn reset_profile_iterator(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<(), DatabaseError> {
        self.send_event_to_concurrent_runner(|s| {
            (
                account_id.as_light(),
                ConcurrentWriteCommand::ResetProfileIterator { s, account_id },
            )
        })
        .await
    }

    async fn send_event<T, R: Into<WriteCommand>>(
        &self,
        get_event: impl FnOnce(ResultSender<T>) -> R,
    ) -> Result<T, DatabaseError> {
        let (result_sender, receiver) = oneshot::channel();
        self.sender
            .send(get_event(result_sender).into())
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
            WriteCommand::Logout { s, account_id } => self.write().logout(account_id).await.send(s),
            WriteCommand::EndConnectionSession { s, account_id } => self
                .write()
                .end_connection_session(account_id, false)
                .await
                .send(s),
            WriteCommand::SetNewAuthPair {
                s,
                account_id,
                pair,
                address,
            } => self
                .write()
                .set_new_auth_pair(account_id, pair, address)
                .await
                .send(s),
            WriteCommand::Account(cmd) => self.handle_account_cmd(cmd).await,
            WriteCommand::Profile(cmd) => self.handle_profile_cmd(cmd).await,
            WriteCommand::Media(cmd) => self.handle_media_cmd(cmd).await,
            WriteCommand::Chat(cmd) => self.handle_chat_cmd(cmd).await,
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
            ConcurrentWriteCommand::NextProfiles { s, account_id } => {
                self.start_cmd_task(p, l, s, move |w| async move {
                    w.user_write_commands_account()
                        .next_profiles(account_id)
                        .await
                })
                .await;
            }
            ConcurrentWriteCommand::ResetProfileIterator { s, account_id } => {
                self.start_cmd_task(p, l, s, move |w| async move {
                    w.user_write_commands_account()
                        .reset_profile_iterator(account_id)
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
