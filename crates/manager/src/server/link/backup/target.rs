use std::{num::Wrapping, sync::Arc, time::Duration};

use backup::{DeleteOldFileBackups, SaveContentBackup, SaveFileBackup};
use error_stack::{FutureExt, Result, ResultExt};
use manager_api::{
    ClientConfig, ManagerClient,
    protocol::{
        ClientConnectionReadSend, ClientConnectionWriteSend, ConnectionUtilsRead,
        ConnectionUtilsWrite,
    },
};
use manager_config::{Config, file::BackupLinkConfigTarget};
use manager_model::{
    AccountAndContent, BackupMessage, BackupMessageType, Sha256Bytes, SourceToTargetMessage,
    TargetToSourceMessage,
};
use simple_backend_utils::{ContextExt, IntoReportFromString};
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::{error, info, warn};

use crate::{
    api::GetConfig,
    server::{ServerQuitWatcher, app::S},
};

mod backup;

#[derive(thiserror::Error, Debug)]
enum BackupTargetError {
    #[error("Reading error")]
    Read,

    #[error("Writing error")]
    Write,

    #[error("Broken message channel")]
    BrokenMessageChannel,

    #[error("Link connection client error")]
    Client,

    #[error("Deserialize")]
    Deserialize,

    #[error("Portocol")]
    Protocol,

    #[error("Invalid account ID")]
    InvalidAccountId,

    #[error("Invalid content ID")]
    InvalidContentId,

    #[error("Invalid file name")]
    InvalidFileName,

    #[error("File overwriting and removing failed")]
    FileOverwritingAndRemovingFailed,

    #[error("Directory removing failed")]
    RemoveDir,

    #[error("File backup already exists")]
    FileBackupAlreadyExists,

    #[error("File backup packet number mismatch")]
    FileBackupPacketNumberMismatch,

    #[error("File backup data corruption detected")]
    FileBackupDataCorruptionDetected,

    #[error("File flush")]
    FileFlush,

    #[error("File sync")]
    FileSync,

    #[error("File rename")]
    FileRename,

    #[error("Time related error")]
    Time,

    #[error("Content data corruption detected")]
    ContentDataCorruptionDetected,
}

#[derive(Debug)]
pub struct BackupLinkManagerTargetQuitHandle {
    task: JoinHandle<()>,
}

impl BackupLinkManagerTargetQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("Backup link manager target quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct BackupLinkManagerTarget {
    state: S,
}

impl BackupLinkManagerTarget {
    pub fn new_manager(
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> BackupLinkManagerTargetQuitHandle {
        let manager = Self {
            state: state.clone(),
        };

        let task = tokio::spawn(manager.run(quit_notification.resubscribe()));

        BackupLinkManagerTargetQuitHandle { task }
    }

    async fn run(self, mut quit_notification: ServerQuitWatcher) {
        if let Some(config) = self.state.config().backup_link().target.clone() {
            tokio::select! {
                _ = self.create_connection_loop(config) => (),
                _ = quit_notification.recv() => (),
            }
        } else {
            let _ = quit_notification.recv().await;
        }
    }

    async fn create_connection_loop(mut self, config: BackupLinkConfigTarget) {
        let mut retry_wait_seconds = 2;
        loop {
            match self.create_connection(&config).await {
                Ok(()) => {
                    info!(
                        "Backup target link disconnected, retrying connection in {} seconds",
                        retry_wait_seconds
                    );
                }
                Err(e) => {
                    error!("Backup target link error: {:?}", e);
                    info!(
                        "Retrying backup target link connection in {} seconds",
                        retry_wait_seconds
                    );
                }
            }
            tokio::time::sleep(Duration::from_secs(retry_wait_seconds)).await;
            retry_wait_seconds = (retry_wait_seconds.pow(2)).min(60 * 60);
        }
    }

    async fn create_connection(
        &mut self,
        config: &BackupLinkConfigTarget,
    ) -> Result<(), BackupTargetError> {
        let client = ManagerClient::connect(ClientConfig {
            url: config.url.clone(),
            tls_config: self.state.config().client_tls_config(),
            api_key: self.state.config().api_key().to_string(),
        })
        .await
        .change_context(BackupTargetError::Client)?;

        let (reader, writer) = client
            .backup_link(config.password.clone())
            .change_context(BackupTargetError::Client)
            .await?;

        info!("Backup target link connected");

        let (sender, receiver) = mpsc::channel(10);

        tokio::select! {
            r = self.send_connection_tests(sender.clone()) => r,
            r = self.handle_reading(reader, sender) => r,
            r = self.handle_writing(writer, receiver) => r,
        }
    }

    async fn send_connection_tests(
        &self,
        sender: mpsc::Sender<BackupMessage>,
    ) -> Result<(), BackupTargetError> {
        loop {
            tokio::time::sleep(Duration::from_secs(60 * 60)).await;
            sender
                .send(BackupMessage::empty())
                .await
                .change_context(BackupTargetError::BrokenMessageChannel)?;
        }
    }

    async fn handle_reading(
        &self,
        mut reader: Box<dyn ClientConnectionReadSend>,
        sender: mpsc::Sender<BackupMessage>,
    ) -> Result<(), BackupTargetError> {
        let mut target_state: Option<BackupTargetState> = None;

        loop {
            let Some(m) = reader
                .receive_backup_link_message()
                .await
                .change_context(BackupTargetError::Read)?
            else {
                return Ok(());
            };

            match m.header.message_type {
                BackupMessageType::Empty => {
                    continue;
                }
                BackupMessageType::StartBackupSession => {
                    let state = BackupTargetState::new(
                        self.state.config_arc().clone(),
                        sender.clone(),
                        m.header.backup_session.0,
                    );
                    target_state = Some(state);
                    continue;
                }
                _ => (),
            }

            let Some(target_state) = &mut target_state else {
                warn!(
                    "Ignoring {:?} message. Backup session not started",
                    m.header.message_type
                );
                continue;
            };

            if m.header.backup_session.0 != target_state.current_backup_session {
                warn!(
                    "Ignoring {:?} message. Backup session mismatch",
                    m.header.message_type
                );
                return Ok(());
            }

            target_state.handle_source_to_target_message(m).await?
        }
    }

    async fn handle_writing(
        &self,
        mut writer: Box<dyn ClientConnectionWriteSend>,
        mut receiver: mpsc::Receiver<BackupMessage>,
    ) -> Result<(), BackupTargetError> {
        loop {
            let message = match receiver.recv().await {
                Some(m) => m,
                None => return Err(BackupTargetError::BrokenMessageChannel.report()),
            };
            writer
                .send_backup_link_message(message)
                .await
                .change_context(BackupTargetError::Write)?;
        }
    }
}

struct BackupTargetState {
    sender: mpsc::Sender<SourceToTargetMessage>,
    current_backup_session: u32,
}

impl BackupTargetState {
    fn new(
        config: Arc<Config>,
        sender: mpsc::Sender<BackupMessage>,
        current_backup_session: u32,
    ) -> Self {
        let (source_sender, source_receiver) = mpsc::channel(10);
        tokio::task::spawn(async move {
            BackupSessionTaskTarget::new(config, sender, source_receiver, current_backup_session)
                .run()
                .await;
        });
        Self {
            sender: source_sender,
            current_backup_session,
        }
    }

    async fn handle_source_to_target_message(
        &mut self,
        m: BackupMessage,
    ) -> Result<(), BackupTargetError> {
        let m = m
            .try_into()
            .into_error_string(BackupTargetError::Deserialize)?;
        self.sender
            .send(m)
            .await
            .change_context(BackupTargetError::Deserialize)?;

        Ok(())
    }
}

struct BackupSessionTaskTarget {
    config: Arc<Config>,
    sender: mpsc::Sender<BackupMessage>,
    receiver: mpsc::Receiver<SourceToTargetMessage>,
    current_backup_session: u32,
    synced_accounts: u64,
    synced_content: u64,
    received_files: u64,
    deleted_files: u64,
}

impl BackupSessionTaskTarget {
    pub fn new(
        config: Arc<Config>,
        sender: mpsc::Sender<BackupMessage>,
        receiver: mpsc::Receiver<SourceToTargetMessage>,
        current_backup_session: u32,
    ) -> Self {
        Self {
            config,
            sender,
            receiver,
            current_backup_session,
            synced_accounts: 0,
            synced_content: 0,
            received_files: 0,
            deleted_files: 0,
        }
    }

    pub async fn run(mut self) {
        info!("Backup session started");
        match self.run_and_result().await {
            Ok(()) => (),
            Err(e) => error!("Backup session error: {:?}", e),
        }
        info!(
            "Backup session completed, accounts: {}, content: {}, files: {}, deleted files: {}",
            self.synced_accounts, self.synced_content, self.received_files, self.deleted_files,
        );
    }

    pub async fn run_and_result(&mut self) -> Result<(), BackupTargetError> {
        let mut backup = SaveContentBackup::new(self.config.clone()).await?;

        loop {
            let m = self.receive_content_list().await?;

            for a in &m {
                let mut content_state = backup.update_account_content_backup(a.account_id).await?;
                for &c in &a.content_ids {
                    if content_state.exists(c) {
                        content_state.mark_as_still_existing(c);
                    } else {
                        self.send_message(TargetToSourceMessage::ContentQuery {
                            account_id: a.account_id,
                            content_id: c,
                        })
                        .await?;
                        let (sha256, data) = self.receive_content().await?;
                        content_state.new_content(c, sha256, data).await?;
                    }
                    self.synced_content += 1;
                }
                content_state.finalize().await?;
                backup.mark_as_still_existing(a.account_id);
                self.synced_accounts += 1;
            }

            if m.is_empty() {
                break;
            }

            self.send_message(TargetToSourceMessage::ContentListSyncDone)
                .await?;
        }

        backup.finalize().await?;

        loop {
            let (sha256, file_name) = self.receive_start_file_backup().await?;
            if file_name.is_empty() {
                break;
            }
            let mut state = SaveFileBackup::new(self.config.clone(), sha256, &file_name).await?;
            loop {
                let (packet_number, data) = self.receive_file_backup_data().await?;
                if data.is_empty() {
                    state.finalize(packet_number).await?;
                    self.received_files += 1;
                    break;
                } else {
                    state.save_packet(packet_number, data).await?;
                }
            }
        }

        self.deleted_files = DeleteOldFileBackups::run(self.config.clone()).await?;

        Ok(())
    }

    pub async fn receive_content_list(
        &mut self,
    ) -> Result<Vec<AccountAndContent>, BackupTargetError> {
        let Some(m) = self.receiver.recv().await else {
            return Err(BackupTargetError::BrokenMessageChannel.report());
        };
        match m {
            SourceToTargetMessage::ContentList { data } => Ok(data),
            _ => Err(BackupTargetError::Protocol.report()),
        }
    }

    pub async fn receive_content(&mut self) -> Result<(Sha256Bytes, Vec<u8>), BackupTargetError> {
        let Some(m) = self.receiver.recv().await else {
            return Err(BackupTargetError::BrokenMessageChannel.report());
        };
        match m {
            SourceToTargetMessage::ContentQueryAnswer { sha256, data } => Ok((sha256, data)),
            _ => Err(BackupTargetError::Protocol.report()),
        }
    }

    pub async fn receive_start_file_backup(
        &mut self,
    ) -> Result<(Sha256Bytes, String), BackupTargetError> {
        let Some(m) = self.receiver.recv().await else {
            return Err(BackupTargetError::BrokenMessageChannel.report());
        };
        match m {
            SourceToTargetMessage::StartFileBackup { sha256, file_name } => Ok((sha256, file_name)),
            _ => Err(BackupTargetError::Protocol.report()),
        }
    }

    pub async fn receive_file_backup_data(
        &mut self,
    ) -> Result<(Wrapping<u32>, Vec<u8>), BackupTargetError> {
        let Some(m) = self.receiver.recv().await else {
            return Err(BackupTargetError::BrokenMessageChannel.report());
        };
        match m {
            SourceToTargetMessage::FileBackupData {
                package_number,
                data,
            } => Ok((package_number, data)),
            _ => Err(BackupTargetError::Protocol.report()),
        }
    }

    pub async fn send_message(
        &mut self,
        message: TargetToSourceMessage,
    ) -> Result<(), BackupTargetError> {
        self.sender
            .send(message.into_message(self.current_backup_session))
            .await
            .change_context(BackupTargetError::BrokenMessageChannel)
    }
}
