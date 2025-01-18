//! Handle software updates

use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
    sync::{atomic::Ordering, Arc},
};

use error_stack::{Result, ResultExt};
use manager_model::{BuildInfo, ResetDataQueryParam, SoftwareInfo, SoftwareOptions};
use tokio::{process::Command, task::JoinHandle};
use tracing::{info, warn};

use super::{
    backend_controller::BackendController,
    reboot::{RebootManagerHandle, REBOOT_ON_NEXT_CHECK},
    ServerQuitWatcher,
};
use crate::{
    config::{file::SoftwareUpdateProviderConfig, Config},
    utils::{ContextExt, InProgressChannel, InProgressReceiver, InProgressSender},
};

#[derive(thiserror::Error, Debug)]
pub enum UpdateError {
    #[error("Update manager related config is missing")]
    UpdateManagerConfigMissing,

    #[error("Process start failed")]
    ProcessStartFailed,

    #[error("Process wait failed")]
    ProcessWaitFailed,

    #[error("Process stdin writing failed")]
    ProcessStdinFailed,

    #[error("Command failed with exit status: {0}")]
    CommandFailed(ExitStatus),

    #[error("Invalid key path")]
    InvalidKeyPath,

    #[error("File copying failed")]
    FileCopyingFailed,

    #[error("File reading failed")]
    FileReadingFailed,

    #[error("File writing failed")]
    FileWritingFailed,

    #[error("File moving failed")]
    FileMovingFailed,

    #[error("File removing failed")]
    FileRemovingFailed,

    #[error("Invalid output")]
    InvalidOutput,

    #[error("Invalid input")]
    InvalidInput,

    #[error("Send message failed")]
    SendMessageFailed,

    #[error("Software updater related config is missing")]
    SoftwareUpdaterConfigMissing,

    #[error("Api request failed")]
    ApiRequest,

    #[error("Reboot failed")]
    RebootFailed,

    #[error("Reset data directory was not directory or does not exist")]
    ResetDataDirectoryWasNotDirectory,

    #[error("Reset data directory missing file name")]
    ResetDataDirectoryNoFileName,

    #[error("Stop backend failed")]
    StopBackendFailed,

    #[error("Start backend failed")]
    StartBackendFailed,
}

#[derive(Debug)]
pub struct UpdateManagerQuitHandle {
    task: JoinHandle<()>,
    // Make sure Receiver works until the manager quits.
    _sender: InProgressSender<UpdateManagerMessage>,
}

impl UpdateManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("Update manager quit failed. Error: {:?}", e);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum UpdateManagerMessage {
    UpdateSoftware {
        force_reboot: bool,
        reset_data: ResetDataQueryParam,
        software: SoftwareOptions,
    },
    RestartBackend {
        reset_data: ResetDataQueryParam,
    },
}

pub struct UpdateManagerHandle {
    sender: InProgressSender<UpdateManagerMessage>,
}

impl UpdateManagerHandle {
    pub async fn send_update_request(
        &self,
        software: SoftwareOptions,
        force_reboot: bool,
        reset_data: ResetDataQueryParam,
    ) -> Result<(), UpdateError> {
        let message = UpdateManagerMessage::UpdateSoftware {
            force_reboot,
            reset_data,
            software,
        };
        self.send_message(message).await
    }

    pub async fn send_restart_backend_request(
        &self,
        reset_data: ResetDataQueryParam,
    ) -> Result<(), UpdateError> {
        let message = UpdateManagerMessage::RestartBackend { reset_data };
        self.send_message(message).await
    }

    async fn send_message(&self, message: UpdateManagerMessage) -> Result<(), UpdateError> {
        self.sender
            .send_message(message)
            .await
            .change_context(UpdateError::SendMessageFailed)
    }
}

#[derive(Debug)]
pub struct UpdateManager {
    config: Arc<Config>,
    receiver: InProgressReceiver<UpdateManagerMessage>,
    reboot_manager_handle: RebootManagerHandle,
}

impl UpdateManager {
    pub fn new_manager(
        config: Arc<Config>,
        quit_notification: ServerQuitWatcher,
        reboot_manager_handle: RebootManagerHandle,
    ) -> (UpdateManagerQuitHandle, UpdateManagerHandle) {
        let (sender, receiver) = InProgressChannel::create();

        let manager = Self {
            config,
            receiver,
            reboot_manager_handle,
        };

        let task = tokio::spawn(manager.run(quit_notification));

        let handle = UpdateManagerHandle {
            sender: sender.clone(),
        };

        let quit_handle = UpdateManagerQuitHandle {
            task,
            _sender: sender,
        };

        (quit_handle, handle)
    }

    pub async fn run(mut self, mut quit_notification: ServerQuitWatcher) {
        loop {
            tokio::select! {
                result = self.receiver.is_new_message_available() => {
                    match result {
                        Ok(()) => (),
                        Err(e) => {
                            warn!("Update manager channel broken. Error: {:?}", e);
                            return;
                        }
                    }

                    let container = self.receiver.lock_message_container().await;

                    match container.get_message() {
                        Some(message) => {
                            self.handle_message(message).await;
                        }
                        None => {
                            warn!("Unexpected empty container");
                        }
                    }
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }

    pub async fn handle_message(&self, message: &UpdateManagerMessage) {
        match message.clone() {
            UpdateManagerMessage::UpdateSoftware {
                force_reboot,
                reset_data,
                software,
            } => match self
                .update_software(force_reboot, reset_data, software)
                .await
            {
                Ok(()) => {
                    info!("Software update finished");
                }
                Err(e) => {
                    warn!("Software update failed. Error: {:?}", e);
                }
            },
            UpdateManagerMessage::RestartBackend { reset_data } => {
                match self.restart_backend(reset_data).await {
                    Ok(()) => {
                        info!("Backend restart finished");
                    }
                    Err(e) => {
                        warn!("Backend restart failed. Error: {:?}", e);
                    }
                }
            }
        }
    }

    pub async fn download_latest_info(
        &self,
        software: SoftwareOptions,
    ) -> Result<BuildInfo, UpdateError> {
        // let api = ApiManager::new(&self.config, &self.api_client);
        // api.get_latest_build_info(software)
        //     .await
        //     .change_context(UpdateError::ApiRequest)
        Ok(BuildInfo::default())
    }

    /// Returns empty BuildInfo if it does not exists.
    pub async fn read_latest_build_info(
        &self,
        _software: SoftwareOptions,
    ) -> Result<BuildInfo, UpdateError> {
        // let update_dir = UpdateDirCreator::create_update_dir_if_needed(&self.config);
        // let current_info =
        //     update_dir.join(BuildDirCreator::build_info_json_name(software.to_str()));
        // self.read_build_info(&current_info).await
        // TODO(prod): Implement?
        Ok(BuildInfo::default())
    }

    /// Returns empty BuildInfo if it does not exists.
    pub async fn read_latest_installed_build_info(
        &self,
        software: SoftwareOptions,
    ) -> Result<BuildInfo, UpdateError> {
        let update_dir = UpdateDirCreator::create_update_dir_if_needed(&self.config);
        let current_info = update_dir.join(UpdateDirCreator::installed_build_info_json_name(
            software.to_str(),
        ));
        self.read_build_info(&current_info).await
    }

    /// Returns empty BuildInfo if it does not exists.
    async fn read_build_info(&self, current_info: &Path) -> Result<BuildInfo, UpdateError> {
        if !current_info.exists() {
            return Ok(BuildInfo::default());
        }

        let current_build_info = tokio::fs::read_to_string(&current_info)
            .await
            .change_context(UpdateError::FileReadingFailed)?;

        let current_build_info =
            serde_json::from_str(&current_build_info).change_context(UpdateError::InvalidInput)?;

        Ok(current_build_info)
    }

    pub async fn download_and_verify_latest_software(
        &self,
        _latest_version: &BuildInfo,
        _software: SoftwareOptions,
    ) -> Result<(), UpdateError> {
        // let encrypted_binary = self.download_latest_encrypted_binary(software).await?;

        // let update_dir = UpdateDirCreator::create_update_dir_if_needed(&self.config);
        // let encrypted_binary_path =
        //     update_dir.join(BuildDirCreator::encrypted_binary_name(software.to_str()));
        // tokio::fs::write(&encrypted_binary_path, encrypted_binary)
        //     .await
        //     .change_context(UpdateError::FileWritingFailed)?;

        // self.import_gpg_key_if_configured().await?;
        // let binary_path = update_dir.join(software.to_str());
        // self.decrypt_encrypted_binary(&encrypted_binary_path, &binary_path)
        //     .await?;

        // let latest_build_info_path =
        //     update_dir.join(BuildDirCreator::build_info_json_name(software.to_str()));
        // tokio::fs::write(
        //     &latest_build_info_path,
        //     serde_json::to_string_pretty(&latest_version)
        //         .change_context(UpdateError::InvalidInput)?,
        // )
        // .await
        // .change_context(UpdateError::FileWritingFailed)?;

        // TODO(prod): Implement dowloading binary from GitHub and verifying
        //             it.

        Ok(())
    }

    pub async fn install_latest_software(
        &self,
        latest_version: &BuildInfo,
        force_reboot: bool,
        reset_data: ResetDataQueryParam,
        software: SoftwareOptions,
    ) -> Result<(), UpdateError> {
        let update_dir = UpdateDirCreator::create_update_dir_if_needed(&self.config);
        let binary_path = update_dir.join(software.to_str());

        let installed_build_info_path = update_dir.join(
            UpdateDirCreator::installed_build_info_json_name(software.to_str()),
        );

        if installed_build_info_path.exists() {
            let installed_old_build_info_path = update_dir.join(
                UpdateDirCreator::installed_old_build_info_json_name(software.to_str()),
            );
            tokio::fs::rename(&installed_build_info_path, &installed_old_build_info_path)
                .await
                .change_context(UpdateError::FileMovingFailed)?;
        }

        self.replace_binary(&binary_path, software).await?;

        tokio::fs::write(
            &installed_build_info_path,
            serde_json::to_string_pretty(&latest_version)
                .change_context(UpdateError::InvalidInput)?,
        )
        .await
        .change_context(UpdateError::FileWritingFailed)?;

        if reset_data.reset_data {
            self.reset_data(software).await?;
        }

        REBOOT_ON_NEXT_CHECK.store(true, Ordering::Relaxed);

        if force_reboot {
            self.reboot_manager_handle
                .reboot_now()
                .await
                .change_context(UpdateError::RebootFailed)?;
            info!("Rebooting now");
        } else {
            info!("Rebooting on next check");
        }

        Ok(())
    }

    pub async fn update_software(
        &self,
        force_reboot: bool,
        reset_data: ResetDataQueryParam,
        software: SoftwareOptions,
    ) -> Result<(), UpdateError> {
        let current_version = self.read_latest_build_info(software).await?;
        let latest_version = self.download_latest_info(software).await?;

        if current_version != latest_version {
            info!(
                "Downloading and verifying software...\n{:#?}",
                latest_version
            );
            self.download_and_verify_latest_software(&latest_version, software)
                .await?;
            info!("Software is now downloaded and verified.");
        } else {
            info!("Downloaded software is up to date.\n{:#?}", current_version);
        }

        let latest_installed_version = self.read_latest_installed_build_info(software).await?;
        if latest_version != latest_installed_version {
            info!("Installing software.\n{:#?}", latest_version);
            self.install_latest_software(&latest_version, force_reboot, reset_data, software)
                .await?;
            info!("Software installation completed.");
        } else {
            info!(
                "Installed software is up to date.\n{:#?}",
                latest_installed_version
            );
        }

        Ok(())
    }

    pub async fn replace_binary(
        &self,
        binary: &Path,
        software: SoftwareOptions,
    ) -> Result<(), UpdateError> {
        let target = match software {
            SoftwareOptions::Backend => self.updater_config()?.backend_install_location.clone(),
        };

        if target.exists() {
            tokio::fs::rename(&target, &target.with_extension("old"))
                .await
                .change_context(UpdateError::FileMovingFailed)?;
        }

        tokio::fs::copy(&binary, &target)
            .await
            .change_context(UpdateError::FileCopyingFailed)?;

        let status = Command::new("chmod")
            .arg("u+x")
            .arg(&target)
            .status()
            .await
            .change_context(UpdateError::ProcessWaitFailed)?;
        if !status.success() {
            return Err(UpdateError::CommandFailed(status))
                .attach_printable("Changing binary permissions failed");
        }

        Ok(())
    }

    pub async fn reset_data(&self, software: SoftwareOptions) -> Result<(), UpdateError> {
        if software != SoftwareOptions::Backend {
            return Ok(());
        }

        let backend_reset_data_dir = match &self.updater_config()?.backend_data_reset_dir {
            Some(dir) => dir,
            None => return Ok(()),
        };

        if !backend_reset_data_dir.is_dir() {
            return Err(UpdateError::ResetDataDirectoryWasNotDirectory)
                .attach_printable(backend_reset_data_dir.display().to_string());
        }

        let mut old_dir_name = backend_reset_data_dir
            .file_name()
            .ok_or(UpdateError::ResetDataDirectoryNoFileName.report())?
            .to_string_lossy()
            .to_string();
        old_dir_name.push_str("-old");
        let old_data_dir = backend_reset_data_dir.with_file_name(old_dir_name);
        if old_data_dir.is_dir() {
            info!(
                "Data reset was requested. Removing existing old data directory {}",
                old_data_dir.display()
            );
            tokio::fs::remove_dir_all(&old_data_dir)
                .await
                .change_context(UpdateError::FileRemovingFailed)
                .attach_printable(old_data_dir.display().to_string())?;
        }

        info!(
            "Data reset was requested. Moving {} to {}",
            backend_reset_data_dir.display(),
            old_data_dir.display()
        );
        tokio::fs::rename(&backend_reset_data_dir, &old_data_dir)
            .await
            .change_context(UpdateError::FileMovingFailed)
            .attach_printable(format!(
                "{} -> {}",
                backend_reset_data_dir.display(),
                old_data_dir.display()
            ))?;

        Ok(())
    }

    pub fn updater_config(&self) -> Result<&SoftwareUpdateProviderConfig, UpdateError> {
        self.config
            .software_update_provider()
            .ok_or(UpdateError::SoftwareUpdaterConfigMissing.into())
    }

    pub async fn restart_backend(
        &self,
        reset_data: ResetDataQueryParam,
    ) -> Result<(), UpdateError> {
        let backend_controller = BackendController::new(&self.config);

        backend_controller
            .stop_backend()
            .await
            .change_context(UpdateError::StopBackendFailed)?;

        if reset_data.reset_data {
            self.reset_data(SoftwareOptions::Backend).await?;
        }

        backend_controller
            .start_backend()
            .await
            .change_context(UpdateError::StartBackendFailed)
    }
}

pub struct UpdateDirCreator;

impl UpdateDirCreator {
    pub fn create_update_dir_if_needed(config: &Config) -> PathBuf {
        let build_dir = config.storage_dir().join("update");

        if !Path::new(&build_dir).exists() {
            info!("Creating update directory");
            match std::fs::create_dir(&build_dir) {
                Ok(()) => {
                    info!("Update directory created");
                }
                Err(e) => {
                    warn!(
                        "Update directory creation failed. Error: {:?}, Directory: {}",
                        e,
                        build_dir.display()
                    );
                }
            }
        }

        build_dir
    }

    pub fn installed_build_info_json_name(binary: &str) -> String {
        format!("{}.json.installed", binary)
    }

    pub fn installed_old_build_info_json_name(binary: &str) -> String {
        format!("{}.json.installed.old", binary)
    }

    pub async fn current_software(config: &Config) -> Result<SoftwareInfo, UpdateError> {
        let update_dir = Self::create_update_dir_if_needed(config);
        let backend_info_path = update_dir.join(Self::installed_build_info_json_name(
            SoftwareOptions::Backend.to_str(),
        ));
        let mut info_vec = Vec::new();

        if backend_info_path.exists() {
            let backend_info = tokio::fs::read_to_string(&backend_info_path)
                .await
                .change_context(UpdateError::FileReadingFailed)?;
            let backend_info =
                serde_json::from_str(&backend_info).change_context(UpdateError::InvalidInput)?;
            info_vec.push(backend_info);
        }

        Ok(SoftwareInfo {
            current_software: info_vec,
        })
    }
}
