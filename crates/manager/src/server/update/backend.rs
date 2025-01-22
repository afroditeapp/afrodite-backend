
use std::{path::Path, process::ExitStatus};

use error_stack::{Result, ResultExt};
use manager_config::file::SoftwareUpdateConfig;
use simple_backend_utils::ContextExt;
use tokio::process::Command;

use tracing::info;

#[derive(thiserror::Error, Debug)]
pub enum BackendUtilsError {
    #[error("Process wait failed")]
    ProcessWaitFailed,

    #[error("Command failed with exit status: {0}")]
    CommandFailed(ExitStatus),

    #[error("File copying failed")]
    FileCopyingFailed,

    #[error("File moving failed")]
    FileMovingFailed,

    #[error("File removing failed")]
    FileRemovingFailed,

    #[error("Reset data directory was not directory or does not exist")]
    ResetDataDirectoryWasNotDirectory,

    #[error("Reset data directory missing file name")]
    ResetDataDirectoryNoFileName,

}

pub struct BackendUtils<'a> {
    pub config: &'a SoftwareUpdateConfig,
}

impl BackendUtils<'_> {
    pub async fn replace_backend_binary(
        &self,
        new_binary: &Path,
    ) -> Result<(), BackendUtilsError> {
        let target = self.config.backend_install_location.clone();

        if target.exists() {
            tokio::fs::rename(&target, &target.with_extension("old"))
                .await
                .change_context(BackendUtilsError::FileMovingFailed)?;
        }

        tokio::fs::copy(&new_binary, &target)
            .await
            .change_context(BackendUtilsError::FileCopyingFailed)?;

        let status = Command::new("chmod")
            .arg("u+x")
            .arg(&target)
            .status()
            .await
            .change_context(BackendUtilsError::ProcessWaitFailed)?;
        if !status.success() {
            return Err(BackendUtilsError::CommandFailed(status))
                .attach_printable("Changing binary permissions failed");
        }

        Ok(())
    }
}

pub async fn reset_backend_data(backend_reset_data_dir: &Path) -> Result<(), BackendUtilsError> {
    if !backend_reset_data_dir.is_dir() {
        return Err(BackendUtilsError::ResetDataDirectoryWasNotDirectory)
            .attach_printable(backend_reset_data_dir.display().to_string());
    }

    let mut old_dir_name = backend_reset_data_dir
        .file_name()
        .ok_or(BackendUtilsError::ResetDataDirectoryNoFileName.report())?
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
            .change_context(BackendUtilsError::FileRemovingFailed)
            .attach_printable(old_data_dir.display().to_string())?;
    }

    info!(
        "Data reset was requested. Moving {} to {}",
        backend_reset_data_dir.display(),
        old_data_dir.display()
    );
    tokio::fs::rename(&backend_reset_data_dir, &old_data_dir)
        .await
        .change_context(BackendUtilsError::FileMovingFailed)
        .attach_printable(format!(
            "{} -> {}",
            backend_reset_data_dir.display(),
            old_data_dir.display()
        ))?;

    Ok(())
}
