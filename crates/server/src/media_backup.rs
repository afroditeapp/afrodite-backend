use std::{
    process::{ExitStatus, Stdio},
    sync::Arc,
    time::Duration,
};

use app_manager::utils::IntoReportExt;

use time::{OffsetDateTime, Time, UtcOffset};
use tokio::{io::AsyncWriteExt, process::Command, sync::mpsc, task::JoinHandle, time::sleep};
use tracing::log::{error, info, warn};

use config::Config;
use model::*;

use crate::{
    app::connection::ServerQuitWatcher,
    data::{file::utils::IMAGE_DIR_NAME, DatabaseRoot},
};

use error_stack::{Result, ResultExt};

pub const MEDIA_BACKUP_MANAGER_QUEUE_SIZE: usize = 64;

#[derive(thiserror::Error, Debug)]
pub enum MediaBackupError {
    #[error("Process start failed")]
    ProcessStart,
    #[error("Process wait failed")]
    ProcessWait,
    #[error("Process stdin missing")]
    ProcessStdinMissing,

    #[error("Invalid process ID")]
    InvalidPid,
    #[error("Sending signal failed")]
    SendSignal,

    #[error("Command failed with exit status: {0}")]
    CommandFailed(ExitStatus),

    #[error("Time error")]
    TimeError,
    #[error("Config error")]
    ConfigError,

    #[error("Database error")]
    DatabaseError,

    #[error("Invalid output")]
    InvalidOutput,
    #[error("Invalid path")]
    InvalidPath,

    #[error("sftp stdin write error")]
    SftpStdinWriteError,
    #[error("sftp stdin close error")]
    SftpStdinCloseError,

    #[error("MediaBackup manager not available")]
    MediaBackupManagerNotAvailable,
}

#[derive(Debug)]
pub struct MediaBackupQuitHandle {
    task: JoinHandle<()>,
    sender: mpsc::Sender<MediaBackupMessage>,
}

impl MediaBackupQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("Media backup manager quit failed. Error: {:?}", e);
            }
        }
    }
}

#[derive(Debug)]
pub enum MediaBackupMessage {
    BackupJpegImage {
        account: AccountIdLight,
        content_id: ContentId,
    },
}

#[derive(Debug, Clone)]
pub struct MediaBackupHandle {
    sender: mpsc::Sender<MediaBackupMessage>,
}

impl MediaBackupHandle {
    pub async fn backup_jpeg_image(
        &self,
        account: AccountIdLight,
        content_id: ContentId,
    ) -> Result<(), MediaBackupError> {
        self.sender
            .send(MediaBackupMessage::BackupJpegImage {
                account,
                content_id,
            })
            .await
            .into_error(MediaBackupError::MediaBackupManagerNotAvailable)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct MediaBackupManager {
    config: Arc<Config>,
    receiver: mpsc::Receiver<MediaBackupMessage>,
}

impl MediaBackupManager {
    pub fn new(
        config: Arc<Config>,
        quit_notification: ServerQuitWatcher,
    ) -> (MediaBackupQuitHandle, MediaBackupHandle) {
        let (sender, receiver) = mpsc::channel(MEDIA_BACKUP_MANAGER_QUEUE_SIZE);

        let manager = Self { config, receiver };

        let task = tokio::spawn(manager.run(quit_notification));

        let handle = MediaBackupHandle { sender };

        let quit_handle = MediaBackupQuitHandle {
            task,
            sender: handle.sender.clone(),
        };

        (quit_handle, handle)
    }

    pub async fn run(mut self, mut quit_notification: ServerQuitWatcher) {
        if self.config.media_backup().is_none() {
            info!("Media backup manager disabled. There is no confguration for it.");
            loop {
                tokio::select! {
                    message = self.receiver.recv() => {
                        match message {
                            Some(_) => (),
                            None => {
                                warn!("Media backup manager channel closed");
                                return;
                            }
                        }
                    }
                    _ = quit_notification.recv() => {
                        return;
                    }
                }
            }
        }

        info!("Media backup manager enabled");

        let mut check_cooldown = false;

        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(120)), if check_cooldown => {
                    check_cooldown = false;
                }
                result = Self::sleep_until(&self.config), if !check_cooldown => {
                    match result {
                        Ok(()) => {
                            self.run_rsync_and_log_error().await;
                        },
                        Err(e) => {
                            warn!("Sleep until failed. Error: {:?}", e);
                        }
                    }
                    check_cooldown = true;
                }
                message = self.receiver.recv() => {
                    match message {
                        Some(message) => {
                            self.handle_message(message).await;
                        }
                        None => {
                            warn!("Media backup manager channel closed");
                            return;
                        }
                    }
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }

    pub async fn handle_message(&self, message: MediaBackupMessage) {
        match message {
            MediaBackupMessage::BackupJpegImage {
                account,
                content_id,
            } => match self.backup_one_image_file(account, content_id).await {
                Ok(()) => {
                    info!(
                        "File backup successful for {} {}",
                        account.as_uuid(),
                        content_id.as_uuid()
                    );
                }
                Err(e) => {
                    warn!("File backup failed. Error: {:?}", e);
                }
            },
        }
    }

    pub async fn run_rsync_and_log_error(&self) {
        info!("Running rsync");
        match self.run_rsync().await {
            Ok(()) => {
                info!("rsync successful");
            }
            Err(e) => {
                warn!("rsync failed. Error: {:?}", e);
            }
        }
    }

    pub async fn run_rsync(&self) -> Result<(), MediaBackupError> {
        let media_config = if let Some(config) = self.config.media_backup() {
            config
        } else {
            return Err(MediaBackupError::ConfigError.into());
        };

        let db_root = DatabaseRoot::new(&self.config.database_dir())
            .change_context(MediaBackupError::DatabaseError)?;
        let files_dir = db_root.file_dir();
        let mut files_dir_string = files_dir.path().to_string_lossy().to_string();
        if !files_dir_string.ends_with("/") {
            files_dir_string = format!("{}/", files_dir_string);
        }

        let target_location = format!(
            "{}@{}:{}",
            &media_config.ssh_address.username,
            &media_config.ssh_address.address,
            &media_config.target_location.to_string_lossy(),
        );

        let status = Command::new("rsync")
            // Archive option
            .arg("-a")
            // Delete files that don't exist in source
            .arg("--delete")
            .arg("--exclude=tmp")
            .arg("-e")
            .arg(format!(
                "ssh -i {}",
                &media_config.ssh_private_key.path.to_string_lossy()
            ))
            // Source directory. Trailing slash is important.
            .arg(files_dir_string)
            // Target directory.
            .arg(target_location)
            .kill_on_drop(true)
            .status()
            .await
            .into_error(MediaBackupError::ProcessStart)?;

        if !status.success() {
            return Err(MediaBackupError::CommandFailed(status).into());
        }

        Ok(())
    }

    pub async fn backup_one_image_file(
        &self,
        account: AccountIdLight,
        content: ContentId,
    ) -> Result<(), MediaBackupError> {
        let db_root = DatabaseRoot::new(&self.config.database_dir())
            .change_context(MediaBackupError::DatabaseError)?;
        let image_file = db_root.file_dir().image_content(account, content);
        let abs_src_file =
            std::fs::canonicalize(image_file.path()).into_error(MediaBackupError::InvalidPath)?;

        let media_config = if let Some(config) = self.config.media_backup() {
            config
        } else {
            return Err(MediaBackupError::ConfigError.into());
        };

        let ssh_key = media_config
            .ssh_private_key
            .path
            .to_string_lossy()
            .to_string();
        let ssh_address = format!(
            "{}@{}",
            &media_config.ssh_address.username, &media_config.ssh_address.address,
        );

        let account_string = account.to_string();

        let commands = format!(
            "@cd '{}'\n\
             -@mkdir '{}'\n\
             @cd '{}'\n\
             -@mkdir '{}'\n\
             @cd '{}'\n\
             @put '{}' '{}'\n\
             @bye\n",
            media_config.target_location.to_string_lossy(),
            &account_string,
            &account_string,
            IMAGE_DIR_NAME,
            IMAGE_DIR_NAME,
            abs_src_file.to_string_lossy(),
            content.jpg_image(),
        );

        let mut process = Command::new("sftp")
            .arg("-b")
            .arg("-")
            .arg("-i")
            .arg(ssh_key)
            .arg(ssh_address)
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .stdin(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .into_error(MediaBackupError::ProcessStart)?;

        let mut stdin = process
            .stdin
            .take()
            .ok_or(MediaBackupError::ProcessStdinMissing)?;

        stdin
            .write_all(commands.as_bytes())
            .await
            .into_error(MediaBackupError::SftpStdinWriteError)?;
        stdin
            .shutdown()
            .await
            .into_error(MediaBackupError::SftpStdinCloseError)?;

        let status = process
            .wait()
            .await
            .into_error(MediaBackupError::ProcessWait)?;

        if !status.success() {
            return Err(MediaBackupError::CommandFailed(status).into());
        }

        Ok(())
    }

    pub async fn sleep_until(config: &Config) -> Result<(), MediaBackupError> {
        let now = Self::get_local_time().await?;

        let target_time = if let Some(config) = config.media_backup() {
            Time::from_hms(config.rsync_time.hours, config.rsync_time.minutes, 0)
                .into_error(MediaBackupError::TimeError)?
        } else {
            futures::future::pending::<()>().await;
            return Err(MediaBackupError::ConfigError.into());
        };

        let target_date_time = now.replace_time(target_time);

        let duration = if target_date_time > now {
            target_date_time - now
        } else {
            let tomorrow = now + Duration::from_secs(24 * 60 * 60);
            let tomorrow_target_date_time = tomorrow.replace_time(target_time);
            tomorrow_target_date_time - now
        };

        sleep(duration.unsigned_abs()).await;

        Ok(())
    }

    pub async fn get_local_time() -> Result<OffsetDateTime, MediaBackupError> {
        let now: OffsetDateTime = OffsetDateTime::now_utc();
        let offset = Self::get_utc_offset_hours().await?;
        let now = now
            .to_offset(UtcOffset::from_hms(offset, 0, 0).into_error(MediaBackupError::TimeError)?);
        Ok(now)
    }

    pub async fn get_utc_offset_hours() -> Result<i8, MediaBackupError> {
        let output = Command::new("date")
            .arg("+%z")
            .output()
            .await
            .into_error(MediaBackupError::ProcessWait)?;

        if !output.status.success() {
            tracing::error!("date command failed");
            return Err(MediaBackupError::CommandFailed(output.status).into());
        }

        let offset =
            std::str::from_utf8(&output.stdout).into_error(MediaBackupError::InvalidOutput)?;

        let multiplier = match offset.chars().nth(0) {
            Some('-') => -1,
            _ => 1,
        };

        let hours = offset
            .chars()
            .skip(1)
            .take(2)
            .collect::<String>()
            .trim_start_matches('0')
            .parse::<i8>()
            .into_error(MediaBackupError::InvalidOutput)?;

        Ok(hours * multiplier)
    }
}
