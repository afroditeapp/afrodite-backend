use std::{
    path::{Component, PathBuf},
    process::{ExitStatus, Stdio},
    sync::Arc,
    time::Duration,
};

use error_stack::{Result, ResultExt};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_database::data::create_dirs_and_get_files_dir_path;
use simple_backend_utils::ContextExt;
use tokio::{io::AsyncWriteExt, process::Command, sync::mpsc, task::JoinHandle, time::sleep};
use tracing::log::{error, info, warn};

use crate::{utils::time::sleep_until_current_time_is_at, ServerQuitWatcher};

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

    #[error("Data error")]
    DataError,

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
    // Make sure that Receiver works until manager quits.
    _sender: mpsc::Sender<MediaBackupMessage>,
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
        /// Path relative to files dir
        image_in_files_dir: PathBuf,
    },
}

#[derive(Debug, Clone)]
pub struct MediaBackupHandle {
    sender: mpsc::Sender<MediaBackupMessage>,
}

impl MediaBackupHandle {
    /// The path must be relative to files dir
    pub async fn backup_jpeg_image(&self, image: PathBuf) -> Result<(), MediaBackupError> {
        self.sender
            .send(MediaBackupMessage::BackupJpegImage {
                image_in_files_dir: image,
            })
            .await
            .change_context(MediaBackupError::MediaBackupManagerNotAvailable)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct MediaBackupManager {
    config: Arc<SimpleBackendConfig>,
    receiver: mpsc::Receiver<MediaBackupMessage>,
}

impl MediaBackupManager {
    pub fn new_manager(
        config: Arc<SimpleBackendConfig>,
        quit_notification: ServerQuitWatcher,
    ) -> (MediaBackupQuitHandle, MediaBackupHandle) {
        let (sender, receiver) = mpsc::channel(MEDIA_BACKUP_MANAGER_QUEUE_SIZE);

        let manager = Self { config, receiver };

        let task = tokio::spawn(manager.run(quit_notification));

        let handle = MediaBackupHandle { sender };

        let quit_handle = MediaBackupQuitHandle {
            task,
            _sender: handle.sender.clone(),
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
            MediaBackupMessage::BackupJpegImage { image_in_files_dir } => {
                match self.backup_one_image_file(image_in_files_dir.clone()).await {
                    Ok(()) => {
                        info!(
                            "File backup successful {}",
                            image_in_files_dir.to_string_lossy()
                        );
                    }
                    Err(e) => {
                        warn!("File backup failed. Error: {:?}", e);
                    }
                }
            }
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
            return Err(MediaBackupError::ConfigError.report());
        };

        let mut files_dir_string = self.file_dir()?.to_string_lossy().to_string();
        if !files_dir_string.ends_with('/') {
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
            .change_context(MediaBackupError::ProcessStart)?;

        if !status.success() {
            return Err(MediaBackupError::CommandFailed(status).report());
        }

        Ok(())
    }

    pub fn file_dir(&self) -> Result<PathBuf, MediaBackupError> {
        let files_dir = create_dirs_and_get_files_dir_path(&self.config)
            .change_context(MediaBackupError::DataError)?;
        Ok(files_dir)
    }

    pub async fn backup_one_image_file(&self, image: PathBuf) -> Result<(), MediaBackupError> {
        let image_file = self.file_dir()?.join(image.clone());
        let abs_src_file =
            std::fs::canonicalize(image_file).change_context(MediaBackupError::InvalidPath)?;

        let media_config = if let Some(config) = self.config.media_backup() {
            config
        } else {
            return Err(MediaBackupError::ConfigError.report());
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

        let mut sftp_commands = SftpCommands::new();
        sftp_commands.add_cd(media_config.target_location.to_string_lossy().as_ref());

        let component_count = image.components().count();
        for (i, component) in image.components().enumerate() {
            if let Component::Normal(dir_or_img) = component {
                let dir_or_img = dir_or_img.to_string_lossy().to_string();
                if i + 1 == component_count {
                    // Last component is image file name.
                    sftp_commands.add_put(abs_src_file.to_string_lossy().as_ref(), &dir_or_img)
                } else {
                    // Create and change directory on remote machine.
                    sftp_commands.add_mkdir(&dir_or_img);
                    sftp_commands.add_cd(&dir_or_img);
                }
            } else {
                return Err(MediaBackupError::InvalidPath.report());
            }
            let component = component.as_os_str().to_string_lossy().to_string();
            sftp_commands.add_mkdir(&component);
            sftp_commands.add_cd(&component);
        }

        sftp_commands.add_bye();

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
            .change_context(MediaBackupError::ProcessStart)?;

        let mut stdin = process
            .stdin
            .take()
            .ok_or(MediaBackupError::ProcessStdinMissing)?;

        stdin
            .write_all(sftp_commands.commands.as_bytes())
            .await
            .change_context(MediaBackupError::SftpStdinWriteError)?;
        stdin
            .shutdown()
            .await
            .change_context(MediaBackupError::SftpStdinCloseError)?;

        let status = process
            .wait()
            .await
            .change_context(MediaBackupError::ProcessWait)?;

        if !status.success() {
            return Err(MediaBackupError::CommandFailed(status).report());
        }

        Ok(())
    }

    pub async fn sleep_until(config: &SimpleBackendConfig) -> Result<(), MediaBackupError> {
        if let Some(config) = config.media_backup() {
            sleep_until_current_time_is_at(config.rsync_time)
                .await
                .change_context(MediaBackupError::TimeError)?;
            Ok(())
        } else {
            futures::future::pending::<()>().await;
            unreachable!()
        }
    }
}

pub struct SftpCommands {
    commands: String,
}

impl SftpCommands {
    pub fn new() -> Self {
        Self {
            commands: String::new(),
        }
    }

    fn add_command(&mut self, command: &str) {
        self.commands.push_str(command);
        self.commands.push('\n');
    }

    pub fn add_mkdir(&mut self, dir: &str) {
        self.add_command(&format!("-@mkdir '{}'", dir));
    }

    pub fn add_cd(&mut self, dir: &str) {
        self.add_command(&format!("@cd '{}'", dir));
    }

    pub fn add_put(&mut self, src: &str, dst: &str) {
        self.add_command(&format!("@put '{}' '{}'", src, dst));
    }

    pub fn add_bye(&mut self) {
        self.add_command("@bye");
    }

    pub fn commands(&self) -> &str {
        &self.commands
    }
}

impl Default for SftpCommands {
    fn default() -> Self {
        Self::new()
    }
}
