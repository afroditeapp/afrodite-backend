
use std::path::Path;

use error_stack::{Result, ResultExt};
use manager_config::file::SoftwareUpdateConfig;
use simple_backend_utils::ContextExt;
use tokio::process::Command;

use tracing::info;

use super::UpdateError;

pub struct BackendUtils<'a> {
    pub config: &'a SoftwareUpdateConfig,
}

impl BackendUtils<'_> {
    pub async fn replace_backend_binary(
        &self,
        new_binary: &Path,
    ) -> Result<(), UpdateError> {
        let target = self.config.backend_install_location.clone();

        if target.exists() {
            tokio::fs::rename(&target, &target.with_extension("old"))
                .await
                .change_context(UpdateError::FileMovingFailed)?;
        }

        tokio::fs::copy(&new_binary, &target)
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

    pub async fn reset_backend_data(&self) -> Result<(), UpdateError> {
        let backend_reset_data_dir = match &self.config.backend_data_reset_dir {
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
}
