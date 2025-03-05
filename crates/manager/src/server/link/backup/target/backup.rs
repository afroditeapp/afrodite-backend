use std::{collections::HashSet, path::{Path, PathBuf}, sync::Arc};

use manager_config::Config;
use simple_backend_utils::{file::overwrite_and_remove_if_exists, ContextExt, IntoReportFromString, UuidBase64Url};
use tracing::warn;

use super::BackupTargetError;

use error_stack::{Result, ResultExt};

const BACKUP_DIR_NAME: &str = "backup";
const CONTENT_DIR_NAME: &str = "content";

struct BackupDirUtils<'a> {
    config: &'a Config,
}

impl<'a> BackupDirUtils<'a> {
    fn new(config: &'a Config) -> Self {
        Self {
            config,
        }
    }

    fn create_dir_if_needed(&self, existing_dir: &Path, dir_name: &str) -> PathBuf {
        let dir = existing_dir.join(dir_name);

        if !Path::new(&dir).exists() {
            match std::fs::create_dir(&dir) {
                Ok(()) => (),
                Err(e) => {
                    warn!(
                        "Directory creation failed. Error: {:?}, Directory: {}",
                        e,
                        dir.display()
                    );
                }
            }
        }

        dir
    }

    fn create_backup_dir_if_needed(&self) -> PathBuf {
        self.create_dir_if_needed(self.config.storage_dir(), BACKUP_DIR_NAME)
    }

    fn create_content_dir_if_needed(&self) -> PathBuf {
        self.create_dir_if_needed(&self.create_backup_dir_if_needed(), CONTENT_DIR_NAME)
    }

    fn create_account_content_dir_if_needed(&self, account: UuidBase64Url) -> PathBuf {
        self.create_dir_if_needed(&self.create_content_dir_if_needed(), &account.to_string())
    }

    fn content_file_path(&self, account: UuidBase64Url, content: UuidBase64Url) -> PathBuf {
        self.create_account_content_dir_if_needed(account).join(content.to_string())
    }
}


pub struct SaveContentBackup {
    config: Arc<Config>,
    initial_accounts: HashSet<UuidBase64Url>,
}

impl SaveContentBackup {
    pub async fn new(
        config: Arc<Config>,
    ) -> Result<Self, BackupTargetError> {
        let dir = BackupDirUtils::new(&config).create_content_dir_if_needed();

        let mut initial_accounts = HashSet::new();

        let mut iterator = tokio::fs::read_dir(dir)
            .await
            .change_context(BackupTargetError::Read)?;

        while let Some(e) = iterator.next_entry().await.change_context(BackupTargetError::Read)? {
            if !e.path().is_dir() {
                continue;
            }

            let name = e.file_name();
            let Some(text) = name.to_str() else {
                return Err(BackupTargetError::InvalidAccountId.report());
            };

            let account_id = UuidBase64Url::from_text(text)
                .into_error_string(BackupTargetError::InvalidAccountId)?;

            initial_accounts.insert(account_id);
        }

        Ok(Self {
            config,
            initial_accounts,
        })
    }

    pub async fn update_account_content_backup(&self, account: UuidBase64Url) -> Result<UpdateAccountContent, BackupTargetError> {
        let dir = BackupDirUtils::new(&self.config).create_account_content_dir_if_needed(account);

        let mut initial_content = HashSet::new();

        let mut iterator = tokio::fs::read_dir(dir)
            .await
            .change_context(BackupTargetError::Read)?;

        while let Some(e) = iterator.next_entry().await.change_context(BackupTargetError::Read)? {
            if !e.path().is_file() {
                continue;
            }

            let name = e.file_name();
            let Some(text) = name.to_str() else {
                return Err(BackupTargetError::InvalidContentId.report());
            };

            let content_id = UuidBase64Url::from_text(text)
                .into_error_string(BackupTargetError::InvalidContentId)?;

            initial_content.insert(content_id);
        }

        Ok(UpdateAccountContent {
            config: self.config.clone(),
            account,
            initial_content,
        })
    }

    pub fn mark_as_still_existing(&mut self, account: UuidBase64Url) {
        self.initial_accounts.remove(&account);
    }

    /// Remove accounts which does not exist anymore
    pub async fn finalize(self) -> Result<(), BackupTargetError> {
        for &a in &self.initial_accounts {
            let update = self.update_account_content_backup(a).await?;
            update.finalize().await?;
            let dir = BackupDirUtils::new(&self.config).create_account_content_dir_if_needed(a);
            tokio::fs::remove_dir(&dir)
                .await
                .change_context(BackupTargetError::RemoveDir)
                .attach_printable_lazy(move || dir.to_string_lossy().to_string())?;
        }

        Ok(())
    }
}


pub struct UpdateAccountContent {
    config: Arc<Config>,
    account: UuidBase64Url,
    initial_content: HashSet<UuidBase64Url>,
}


impl UpdateAccountContent {
    pub fn exists(&self, content: UuidBase64Url) -> bool {
        BackupDirUtils::new(&self.config)
            .content_file_path(self.account, content)
            .exists()
    }

    pub fn mark_as_still_existing(&mut self, content: UuidBase64Url) {
        self.initial_content.remove(&content);
    }

    pub async fn new_content(&self, content: UuidBase64Url, data: Vec<u8>) -> Result<(), BackupTargetError> {
        let f = BackupDirUtils::new(&self.config)
            .content_file_path(self.account, content);
        tokio::fs::write(&f, data)
            .await
            .change_context( BackupTargetError::Write)
    }

    pub async fn finalize(self) -> Result<(), BackupTargetError> {
        for c in self.initial_content {
            let f = BackupDirUtils::new(&self.config)
                .content_file_path(self.account, c);
            overwrite_and_remove_if_exists(&f)
                .await
                .change_context(BackupTargetError::FileOverwritingAndRemovingFailed)?;
        }

        Ok(())
    }
}
