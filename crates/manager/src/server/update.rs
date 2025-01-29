//! Handle software updates

use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
    sync::Arc,
};

use archive::extract_backend_binary;
use backend::BackendUtils;
use error_stack::{report, Result, ResultExt};
use github::GitHubApi;
use manager_model::{SoftwareInfo, SoftwareUpdateState, SoftwareUpdateStatus, SoftwareUpdateTaskType};
use sha2::Digest;
use simple_backend_utils::ContextExt;
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::{info, warn, error};

use super::{
    app::S, ServerQuitWatcher
};
use crate::{
    api::GetConfig, utils::{InProgressChannel, InProgressReceiver, InProgressSender}
};
use manager_config::{file::SoftwareUpdateConfig, Config};

pub mod archive;
pub mod github;
pub mod backend;

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


    #[error("GitHub API related error")]
    GitHubApi,

    #[error("Software download failed. More than one matching file name found.")]
    SotwareDownloadFailedAmbiguousFileName,

    #[error("Latest software with matching file name not found from GitHub")]
    SoftwareDownloadFailedNoMatchingFile,

    #[error("Software download failed. Unknown file uploader.")]
    SotwareDownloadFailedUnknownFileUploader,

    #[error("Software downaload failed")]
    SoftwareDownloadFailed,

    #[error("Blocking task failed")]
    BlockingTaskFailed,

    #[error("Serialization failed")]
    Serialize,

    #[error("Selected backend version not found")]
    SelectedVersionNotFound,

    #[error("Backend utils error")]
    BackendUtils,

    #[error("Archive error")]
    Archive,

    #[error("Backend is not found from the archive")]
    ArchiveBackendNotFound,

    #[error("Multiple matching files in the archive")]
    ArchiveMultipleMatchingFiles,
}

#[derive(Debug)]
pub struct UpdateManagerQuitHandle {
    task: JoinHandle<()>,
    // Make sure Receiver works until the manager quits.
    _sender: InProgressSender<SoftwareUpdateTaskType>,
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

#[derive(Debug)]
pub struct UpdateManagerHandle {
    sender: InProgressSender<SoftwareUpdateTaskType>,
    state: Arc<Mutex<SoftwareUpdateStatus>>,
}

impl UpdateManagerHandle {
    pub async fn send_message(&self, message: SoftwareUpdateTaskType) -> Result<(), UpdateError> {
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
    sender: InProgressSender<SoftwareUpdateTaskType>,
    receiver: InProgressReceiver<SoftwareUpdateTaskType>,
    state: Arc<Mutex<SoftwareUpdateStatus>>,
}

#[derive(Debug)]
pub struct UpdateManager {
    receiver: InProgressReceiver<SoftwareUpdateTaskType>,
    internal_state: Arc<Mutex<SoftwareUpdateStatus>>,
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
        let quit_handle_sender = internal_state.sender;
        let manager = Self {
            internal_state: internal_state.state,
            receiver: internal_state.receiver,
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
        let manager = self.state.config()
            .software_update_provider()
            .cloned()
            .map(|config| {
                UpdateManagerInternal {
                    user_agent: self.state.config().update_manager_user_agent(),
                    internal_state: self.internal_state,
                    state: self.state,
                    client: self.client,
                    config: config.clone(),
                }
            });

        if let Some(manager) = &manager {
            match manager.init().await {
                Ok(()) => (),
                Err(e) => {
                    error!("Update manager init failed: {:?}", e);
                }
            }
        }

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
                            if let Some(manager) = &manager {
                                manager.handle_message(message, ).await;
                            } else {
                                warn!("Skipping message {:?}, update manager is not enabled", message);
                            }
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
}

#[derive(Debug)]
pub struct UpdateManagerInternal {
    internal_state: Arc<Mutex<SoftwareUpdateStatus>>,
    state: S,
    client: reqwest::Client,
    config: SoftwareUpdateConfig,
    user_agent: String,
}

impl UpdateManagerInternal {
    async fn init(
        &self,
    ) -> Result<(), UpdateError> {
        let downloaded = self.update_dir().downloaded_backend_info().await?;
        let installed = self.update_dir().installed_backend_info().await?;
        let mut state = self.internal_state.lock().await;
        state.downloaded = downloaded;
        state.installed = installed;
        Ok(())
    }

    async fn handle_message(&self, message: &SoftwareUpdateTaskType) {
        let result = match message.clone() {
            SoftwareUpdateTaskType::Download =>
                self.software_download().await,
            SoftwareUpdateTaskType::Install(info) =>
                self.software_install(info).await,
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

    async fn software_download(
        &self,
    ) -> Result<(), UpdateError> {
        self.set_internal_state_to(SoftwareUpdateState::Downloading).await;
        let r = self.software_download_impl().await;
        self.set_internal_state_to(SoftwareUpdateState::Idle).await;
        r
    }

    async fn software_install(
        &self,
        info: SoftwareInfo,
    ) -> Result<(), UpdateError> {
        self.set_internal_state_to(SoftwareUpdateState::Installing).await;
        let r = self.software_install_impl(info).await;
        self.set_internal_state_to(SoftwareUpdateState::Idle).await;
        r
    }

    async fn set_internal_state_to(&self, new_state: SoftwareUpdateState) {
        let mut state = self.internal_state.lock().await;
        state.state = new_state;
    }

    async fn software_download_impl(
        &self,
    ) -> Result<(), UpdateError> {
        let github_api = GitHubApi {
            client: &self.client,
            updater_config: &self.config,
            user_agent: &self.user_agent,
        };

        let Some(asset) = github_api.get_latest_release_asset().await? else {
            return Err(report!(UpdateError::SoftwareDownloadFailedNoMatchingFile));
        };

        if let Some(downloaded) = self.update_dir().downloaded_backend_info().await? {
            if downloaded.name == asset.name {
                info!("Already downloaded");
                return Ok(());
            }
        }

        self.update_dir().remove_downloaded_backend_and_info_json().await?;

        self.internal_state.lock().await.downloaded = None;

        github_api.download_asset(
            &asset,
            self.update_dir().downloaded_backend_path(),
        ).await?;

        let sha256 = self.update_dir().calculate_backend_sha256().await?;

        let info = SoftwareInfo {
            name: asset.name,
            sha256,
        };

        UpdateDirUtils::save_info_json(
            &info,
            self.update_dir().downloaded_backend_info_json_path(),
        ).await?;

        self.internal_state.lock().await.downloaded = Some(info);

        Ok(())
    }

    async fn software_install_impl(
        &self,
        info: SoftwareInfo,
    ) -> Result<(), UpdateError> {
        if let Some(installed) = self.update_dir().installed_backend_info().await? {
            if info == installed {
                info!("Already installed");
                return Ok(());
            }
        }

        let Some(downloaded) = self.update_dir().downloaded_backend_info().await? else {
            return Err(UpdateError::SelectedVersionNotFound.report());
        };

        if info != downloaded {
            return Err(UpdateError::SelectedVersionNotFound.report());
        }

        let backend_binary = if let Some(archive_file_path) = &self.config.github.archive_backend_binary_path {
            let extracted = self.update_dir().extracted_backend_path();
            extract_backend_binary(
                self.update_dir().downloaded_backend_path(),
                archive_file_path.clone(),
                extracted.clone(),
            ).await?;
            extracted
        } else {
            self.update_dir().downloaded_backend_path()
        };

        self.backend_utils().replace_backend_binary(
            &backend_binary
        )
            .await
            .change_context(UpdateError::BackendUtils)?;

        UpdateDirUtils::save_info_json(
            &info,
            self.update_dir().installed_backend_info_json_path(),
        ).await?;

        self.internal_state.lock().await.installed = Some(info);

        Ok(())
    }

    fn backend_utils(&self) -> BackendUtils {
        BackendUtils {
            config: &self.config,
        }
    }

    fn update_dir(&self) -> UpdateDirUtils {
        UpdateDirUtils {
            config: self.state.config(),
        }
    }
}

struct UpdateDirUtils<'a> {
    config: &'a Config,
}

impl UpdateDirUtils<'_> {
    fn create_update_dir_if_needed(&self) -> PathBuf {
        let dir = self.config.storage_dir().join("update");

        if !Path::new(&dir).exists() {
            info!("Creating update directory");
            match std::fs::create_dir(&dir) {
                Ok(()) => {
                    info!("Update directory created");
                }
                Err(e) => {
                    warn!(
                        "Update directory creation failed. Error: {:?}, Directory: {}",
                        e,
                        dir.display()
                    );
                }
            }
        }

        dir
    }

    fn downloaded_backend_path(&self) -> PathBuf {
        self.create_update_dir_if_needed()
            .join("downloaded_backend")
    }

    fn downloaded_backend_info_json_path(&self) -> PathBuf {
        self.create_update_dir_if_needed()
            .join("downloaded_backend.json")
    }

    fn extracted_backend_path(&self) -> PathBuf {
        self.create_update_dir_if_needed()
            .join("extracted_backend")
    }

    fn installed_backend_info_json_path(&self) -> PathBuf {
        self.create_update_dir_if_needed()
            .join("installed_backend.json")
    }

    pub async fn downloaded_backend_info(&self) -> Result<Option<SoftwareInfo>, UpdateError> {
        Self::read_and_parse_info(
            self.downloaded_backend_info_json_path()
        ).await
    }

    pub async fn installed_backend_info(&self) -> Result<Option<SoftwareInfo>, UpdateError> {
        Self::read_and_parse_info(
            self.installed_backend_info_json_path()
        ).await
    }

    async fn read_and_parse_info(path: PathBuf) -> Result<Option<SoftwareInfo>, UpdateError> {
        if !path.exists() {
            return Ok(None);
        }

        let info = tokio::fs::read_to_string(path)
            .await
            .change_context(UpdateError::FileReadingFailed)?;
        let info = serde_json::from_str(&info)
            .change_context(UpdateError::InvalidInput)?;
        Ok(Some(info))
    }

    pub async fn remove_downloaded_backend_and_info_json(
        &self,
    ) -> Result<(), UpdateError> {
        let downloaded = self.downloaded_backend_path();
        if downloaded.exists() {
            tokio::fs::remove_file(self.downloaded_backend_path())
                .await
                .change_context(UpdateError::FileRemovingFailed)?;
        }

        let info = self.downloaded_backend_info_json_path();
        if info.exists() {
            tokio::fs::remove_file(info)
                .await
                .change_context(UpdateError::FileRemovingFailed)?;
        }

        Ok(())
    }

    pub async fn save_info_json(
        info: &SoftwareInfo,
        path: impl AsRef<Path>,
    ) -> Result<(), UpdateError> {
        let serialized = serde_json::to_string_pretty(info)
            .change_context(UpdateError::Serialize)?;
        tokio::fs::write(path, serialized)
            .await
            .change_context(UpdateError::FileWritingFailed)?;
        Ok(())
    }

    async fn calculate_backend_sha256(&self) -> Result<String, UpdateError> {
        let file = self.downloaded_backend_path();
        tokio::task::spawn_blocking(move || {
            let mut file = std::fs::File::open(file)
                .change_context(UpdateError::FileReadingFailed)?;
            let mut hasher = sha2::Sha256::new();
            std::io::copy(&mut file, &mut hasher)
                .change_context(UpdateError::FileReadingFailed)?;
            let hash = hasher.finalize();
            let hash_string = base16ct::lower::encode_string(&hash);
            Ok(hash_string)
        })
            .await
            .change_context(UpdateError::BlockingTaskFailed)?
    }
}
