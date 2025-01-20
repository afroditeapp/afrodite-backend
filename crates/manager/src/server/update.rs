//! Handle software updates

use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
    sync::Arc,
};

use error_stack::{report, Result, ResultExt};
use manager_model::{BuildInfo, ResetDataQueryParam, SoftwareInfo, SoftwareInfoNew, SoftwareOptions, SoftwareUpdateState, SoftwareUpdateStatus};
use reqwest::{header::{ACCEPT, USER_AGENT}, StatusCode};
use serde_json::Value;
use tokio::{process::Command, sync::Mutex, task::JoinHandle};
use tracing::{info, warn, error};

use super::{
    app::S, backend_controller::BackendController, ServerQuitWatcher
};
use crate::{
    api::GetConfig, config::{file::SoftwareUpdateConfig, Config}, utils::{ContextExt, InProgressChannel, InProgressReceiver, InProgressSender}
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

    #[error("Reset data directory was not directory or does not exist")]
    ResetDataDirectoryWasNotDirectory,

    #[error("Reset data directory missing file name")]
    ResetDataDirectoryNoFileName,

    #[error("Stop backend failed")]
    StopBackendFailed,

    #[error("Start backend failed")]
    StartBackendFailed,

    #[error("GitHub API related error")]
    GitHubApi,

    #[error("Software download failed. More than one matching file name found.")]
    SotwareDownloadFailedAmbiguousFileName,

    #[error("Latest software with matching file name not found from GitHub")]
    SoftwareDownloadFailedNoMatchingFile,
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
    SoftwareDownload,
    SoftwareInstall(SoftwareInfoNew),
    BackendRestart,
    BackendResetData,
}

#[derive(Debug)]
pub struct UpdateManagerHandle {
    sender: InProgressSender<UpdateManagerMessage>,
    state: Arc<Mutex<SoftwareUpdateStatus>>,
}

impl UpdateManagerHandle {
    pub async fn send_message(&self, message: UpdateManagerMessage) -> Result<(), UpdateError> {
        self.sender
            .send_message(message)
            .await
            .change_context(UpdateError::SendMessageFailed)
    }

    pub async fn read_state(&self) -> SoftwareUpdateStatus {
        self.state
            .lock()
            .await
            .clone()
    }
}

#[derive(Debug)]
pub struct UpdateManagerInternalState {
    sender: InProgressSender<UpdateManagerMessage>,
    receiver: InProgressReceiver<UpdateManagerMessage>,
    state: Arc<Mutex<SoftwareUpdateStatus>>,
}

#[derive(Debug)]
pub struct UpdateManager {
    internal_state: UpdateManagerInternalState,
    state: S,
    client: reqwest::Client,
}

impl UpdateManager {
    pub fn new_channel() -> (UpdateManagerHandle, UpdateManagerInternalState) {
        let (sender, receiver) = InProgressChannel::create();
        let state = Arc::new(Mutex::new(SoftwareUpdateStatus::new_idle()));

        let handle = UpdateManagerHandle {
            sender: sender.clone(),
            state: state.clone(),
        };

        let receiver = UpdateManagerInternalState {
            sender,
            receiver,
            state,
        };

        (handle, receiver)
    }

    pub fn new_manager(
        internal_state: UpdateManagerInternalState,
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> UpdateManagerQuitHandle {
        let quit_handle_sender = internal_state.sender.clone();
        let manager = Self {
            internal_state,
            state,
            client: reqwest::Client::new(),
        };

        let task = tokio::spawn(manager.run(quit_notification));

        UpdateManagerQuitHandle {
            task,
            _sender: quit_handle_sender,
        }
    }

    pub async fn run(mut self, mut quit_notification: ServerQuitWatcher) {
        loop {
            tokio::select! {
                result = self.internal_state.receiver.is_new_message_available() => {
                    match result {
                        Ok(()) => (),
                        Err(e) => {
                            warn!("Update manager channel broken. Error: {:?}", e);
                            return;
                        }
                    }

                    let container = self.internal_state.receiver.lock_message_container().await;

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

    async fn handle_message(&self, message: &UpdateManagerMessage) {
        let result = match message.clone() {
            UpdateManagerMessage::SoftwareDownload =>
                self.software_download().await,
            UpdateManagerMessage::SoftwareInstall(info) =>
                self.software_install(info).await,
            UpdateManagerMessage::BackendRestart =>
                self.backend_restart_and_optional_data_reset(false).await,
            UpdateManagerMessage::BackendResetData =>
                self.backend_restart_and_optional_data_reset(true).await,
        };

        match result {
            Ok(()) => {
                info!("Action {:?} completed", message);
            }
            Err(e) => {
                warn!("Action {:?} failed. Error: {:?}", message, e);
            }
        }
    }

    async fn set_internal_state_to(&self, new_state: SoftwareUpdateState) {
        let mut state = self.internal_state.state.lock().await;
        state.state = new_state;
    }

    async fn software_download(
        &self,
    ) -> Result<(), UpdateError> {
        self.set_internal_state_to(SoftwareUpdateState::Downloading).await;
        let r = self.software_download_impl().await;
        self.set_internal_state_to(SoftwareUpdateState::Idle).await;
        r
    }

    async fn software_download_impl(
        &self,
    ) -> Result<(), UpdateError> {
        let Some((file_name, url)) = self.get_latest_release_file_name_and_url().await? else {
            return Err(report!(UpdateError::SoftwareDownloadFailedNoMatchingFile));
        };

        Ok(())
    }

    async fn software_install(
        &self,
        info: SoftwareInfoNew,
    ) -> Result<(), UpdateError> {
        self.set_internal_state_to(SoftwareUpdateState::Installing).await;
        let r = self.software_install_impl(info).await;
        self.set_internal_state_to(SoftwareUpdateState::Idle).await;
        r
    }

    async fn software_install_impl(
        &self,
        info: SoftwareInfoNew,
    ) -> Result<(), UpdateError> {

        Ok(())
    }

    async fn backend_restart_and_optional_data_reset(
        &self,
        data_reset: bool,
    ) -> Result<(), UpdateError> {
        let backend_controller = BackendController::new(self.state.config());

        backend_controller
            .stop_backend()
            .await
            .change_context(UpdateError::StopBackendFailed)?;

        if data_reset {
            self.reset_data(SoftwareOptions::Backend).await?;
        }

        backend_controller
            .start_backend()
            .await
            .change_context(UpdateError::StartBackendFailed)
    }

    /// Returns empty BuildInfo if it does not exists.
    async fn read_latest_installed_build_info(
        &self,
        software: SoftwareOptions,
    ) -> Result<BuildInfo, UpdateError> {
        let update_dir = UpdateDirCreator::create_update_dir_if_needed(self.state.config());
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

    async fn download_and_verify_latest_software(
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

    async fn install_latest_software(
        &self,
        latest_version: &BuildInfo,
        force_reboot: bool,
        reset_data: ResetDataQueryParam,
        software: SoftwareOptions,
    ) -> Result<(), UpdateError> {
        let update_dir = UpdateDirCreator::create_update_dir_if_needed(self.state.config());
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

        Ok(())
    }

    async fn update_software(
        &self,
        force_reboot: bool,
        reset_data: ResetDataQueryParam,
        software: SoftwareOptions,
    ) -> Result<(), UpdateError> {
        // let current_version = self.read_latest_build_info(software).await?;
        // let latest_version = self.download_latest_info(software).await?;

        // if current_version != latest_version {
        //     info!(
        //         "Downloading and verifying software...\n{:#?}",
        //         latest_version
        //     );
        //     self.download_and_verify_latest_software(&latest_version, software)
        //         .await?;
        //     info!("Software is now downloaded and verified.");
        // } else {
        //     info!("Downloaded software is up to date.\n{:#?}", current_version);
        // }

        // let latest_installed_version = self.read_latest_installed_build_info(software).await?;
        // if latest_version != latest_installed_version {
        //     info!("Installing software.\n{:#?}", latest_version);
        //     self.install_latest_software(&latest_version, force_reboot, reset_data, software)
        //         .await?;
        //     info!("Software installation completed.");
        // } else {
        //     info!(
        //         "Installed software is up to date.\n{:#?}",
        //         latest_installed_version
        //     );
        // }

        Ok(())
    }

    async fn replace_binary(
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

    async fn reset_data(&self, software: SoftwareOptions) -> Result<(), UpdateError> {
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

    fn updater_config(&self) -> Result<&SoftwareUpdateConfig, UpdateError> {
        self.state.config()
            .software_update_provider()
            .ok_or(UpdateError::SoftwareUpdaterConfigMissing.into())
    }

    async fn get_latest_release_file_name_and_url(&self) -> Result<Option<(String, String)>, UpdateError> {
        let config = self.updater_config()?;

        let url = format!("https://api.github.com/repos/{}/{}/releases/latest", config.github.owner, config.github.repository);
        let user_agent = format!("{}/{}", self.state.config().backend_pkg_name(), self.state.config().backend_semver_version());
        let request = self.client.get(url)
            .header(ACCEPT, "application/vnd.github+json")
            .header(USER_AGENT, user_agent)
            .header("X-GitHub-Api-Version", "2022-11-28");

        let request = if let Some(token) = config.github.token.clone() {
            request.bearer_auth(token)
        } else {
            request
        };

        let request = request.build()
            .change_context(UpdateError::GitHubApi)?;

        let response = self.client.execute(request)
            .await
            .change_context(UpdateError::GitHubApi)?;

        let status = response.status();
        if status != StatusCode::OK {
            let text = response.text()
                .await
                .change_context(UpdateError::GitHubApi)?;

            return Err(
                report!(UpdateError::GitHubApi)
                    .attach_printable(status)
                    .attach_printable(text)
            );
        }

        let json: Value = response.json()
            .await
            .change_context(UpdateError::GitHubApi)?;

        let assets = json
            .get("assets")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or_default();

        let mut selected_download: Option<(String, String)> = None;
        for a in assets {
            let Some(name) = a.as_object()
                .and_then(|v| v.get("name"))
                .and_then(|v| v.as_str()) else {
                    return Err(report!(UpdateError::GitHubApi));
                };
            let Some(download_url) = a.as_object()
                .and_then(|v| v.get("browser_download_url"))
                .and_then(|v| v.as_str()) else {
                    return Err(report!(UpdateError::GitHubApi));
                };

            if name.ends_with(&config.github.file_name_ending) {
                if let Some((selected_name, _)) = selected_download {
                    return Err(
                        report!(UpdateError::SotwareDownloadFailedAmbiguousFileName)
                            .attach_printable(selected_name.to_string())
                            .attach_printable(name.to_string())
                    );
                } else {
                    selected_download = Some((name.to_string(), download_url.to_string()));
                }
            }
        }

        Ok(selected_download)
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
