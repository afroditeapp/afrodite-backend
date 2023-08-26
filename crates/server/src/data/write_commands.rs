//! Database writing commands
//!

use std::{future::Future, sync::Arc};

use error_stack::{Result, ResultExt};
use model::AccountId;
use tokio::sync::{mpsc, Mutex, OwnedMutexGuard};


use super::{
    write_concurrent::{ConcurrentWriteCommandHandle, ConcurrentWriteHandle},
    RouterDatabaseWriteHandle, SyncWriteHandle,
};
use crate::data::DataError;

pub type WriteCmds = Cmds;

/// Make VSCode rust-analyzer code type annotation shorter.
/// The annotation is displayed when calling write() method.
pub struct Cmds {
    pub write: OwnedMutexGuard<SyncWriteHandle>,
}

impl std::ops::Deref for Cmds {
    type Target = OwnedMutexGuard<SyncWriteHandle>;

    fn deref(&self) -> &Self::Target {
        &self.write
    }
}

#[derive(Debug)]
pub struct WriteCommandRunnerHandle {
    quit_lock: mpsc::Sender<()>,
    sync_write_mutex: Arc<Mutex<SyncWriteHandle>>,
    concurrent_write: ConcurrentWriteCommandHandle,
}

impl WriteCommandRunnerHandle {
    pub fn new(write: RouterDatabaseWriteHandle) -> (Self, WriteCmdWatcher) {
        let (quit_lock, quit_handle) = mpsc::channel::<()>(1);

        let cmd_watcher = WriteCmdWatcher::new(quit_handle);

        let runner_handle = Self {
            quit_lock,
            sync_write_mutex: Mutex::new(write.clone().into_sync_handle()).into(),
            concurrent_write: ConcurrentWriteCommandHandle::new(write.clone()),
        };
        (runner_handle, cmd_watcher)
    }

    pub async fn write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = Result<CmdResult, DataError>> + Send,
        GetCmd: FnOnce(WriteCmds) -> Cmd + Send + 'static,
    >(
        &self,
        write_cmd: GetCmd,
    ) -> Result<CmdResult, DataError> {
        let quit_lock = self.quit_lock.clone();
        let lock = self.sync_write_mutex.clone().lock_owned().await;
        let handle = tokio::spawn(async move {
            let result = write_cmd(Cmds { write: lock }).await;
            drop(quit_lock); // Write completed, so release the quit lock.
            result
        });

        handle
            .await
            .change_context(DataError::CommandResultReceivingFailed)?
    }

    pub async fn concurrent_write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = Result<CmdResult, DataError>> + Send,
        GetCmd: FnOnce(ConcurrentWriteHandle) -> Cmd + Send + 'static,
    >(
        &self,
        account: AccountId,
        write_cmd: GetCmd,
    ) -> Result<CmdResult, DataError> {
        let quit_lock = self.quit_lock.clone();
        let lock = self.concurrent_write.accquire(account).await;
        let handle = tokio::spawn(async move {
            let result = write_cmd(lock).await;
            drop(quit_lock); // Write completed, so release the quit lock.
            result
        });

        handle
            .await
            .change_context(DataError::CommandResultReceivingFailed)?
    }
}

pub struct WriteCmdWatcher {
    receiver: mpsc::Receiver<()>,
}

impl WriteCmdWatcher {
    pub fn new(receiver: mpsc::Receiver<()>) -> Self {
        Self { receiver }
    }

    pub async fn wait_untill_all_writing_ends(mut self) {
        loop {
            match self.receiver.recv().await {
                Some(_) => (),
                None => break,
            }
        }
    }
}
