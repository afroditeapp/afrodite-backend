use std::{collections::HashSet, num::Wrapping, path::{Path, PathBuf}, sync::Arc, time::SystemTime};

use chrono::Utc;
use manager_config::Config;
use manager_model::Sha256Bytes;
use sha2::{Digest, Sha256};
use simple_backend_model::UnixTime;
use simple_backend_utils::{file::overwrite_and_remove_if_exists, ContextExt, IntoReportFromString, UuidBase64Url};
use tokio::io::AsyncWriteExt;
use tracing::warn;

use super::BackupTargetError;

use error_stack::{Result, ResultExt};

const BACKUP_DIR_NAME: &str = "backup";
const CONTENT_DIR_NAME: &str = "content";
const FILES_DIR_NAME: &str = "files";

const TMP_FILE: &str = "file.tmp";

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

    fn content_file_checksum_path(&self, account: UuidBase64Url, content: UuidBase64Url) -> PathBuf {
        self.create_account_content_dir_if_needed(account).join(format!("{}.sha256", content))
    }

    fn create_files_dir_if_needed(&self) -> PathBuf {
        self.create_dir_if_needed(&self.create_backup_dir_if_needed(), FILES_DIR_NAME)
    }

    fn file_path(&self, file: &str) -> PathBuf {
        self.create_files_dir_if_needed().join(file)
    }

    async fn remove_tmp_file_and_get_tmp_file_path(&self) -> Result<PathBuf, BackupTargetError> {
        let path = self.create_backup_dir_if_needed().join(TMP_FILE);
        if path.exists() {
            overwrite_and_remove_if_exists(&path)
                .await
                .change_context(BackupTargetError::FileOverwritingAndRemovingFailed)?;
        }
        Ok(path)
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

    pub async fn new_content(&self, content: UuidBase64Url, sha256: Sha256Bytes, data: Vec<u8>) -> Result<(), BackupTargetError> {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let result = hasher.finalize();
        if result.as_slice() != sha256.0 {
            return Err(BackupTargetError::ContentDataCorruptionDetected.report())
        }

        let f = BackupDirUtils::new(&self.config)
            .content_file_checksum_path(self.account, content);
        tokio::fs::write(f, sha256.to_shasum_tool_compatible_checksum(&content.to_string()))
            .await
            .change_context( BackupTargetError::Write)?;
        let f = BackupDirUtils::new(&self.config)
            .content_file_path(self.account, content);
        tokio::fs::write(f, data)
            .await
            .change_context( BackupTargetError::Write)
    }

    pub async fn finalize(self) -> Result<(), BackupTargetError> {
        for c in self.initial_content {
            let f = BackupDirUtils::new(&self.config)
                .content_file_checksum_path(self.account, c);
            overwrite_and_remove_if_exists(f)
                .await
                .change_context(BackupTargetError::FileOverwritingAndRemovingFailed)?;
            let f = BackupDirUtils::new(&self.config)
                .content_file_path(self.account, c);
            overwrite_and_remove_if_exists(f)
                .await
                .change_context(BackupTargetError::FileOverwritingAndRemovingFailed)?;
        }

        Ok(())
    }
}

pub struct SaveFileBackup {
    expected_packet_number: Wrapping<u32>,
    target_path: PathBuf,
    target_checksum_path: PathBuf,
    target_checksum_file_content: String,
    tmp_file_path: PathBuf,
    tmp_file: tokio::fs::File,
    expected_sha256: Sha256Bytes,
    sha256_state: Sha256,
}

impl SaveFileBackup {
    pub async fn new(
        config: Arc<Config>,
        expected_sha256: Sha256Bytes,
        backup_name: &str,
    ) -> Result<Self, BackupTargetError> {
        let tmp_file_path = BackupDirUtils::new(&config)
            .remove_tmp_file_and_get_tmp_file_path()
            .await?;
        let tmp_file = tokio::fs::File::create(&tmp_file_path)
            .await
            .change_context(BackupTargetError::Write)?;

        let time = Utc::now().format("%Y-%m-%d_%H-%M-%S");
        let name = format!("backup_{}_{}", backup_name, time);
        let target_path = BackupDirUtils::new(&config)
            .file_path(&name);

        if target_path.exists() {
            return Err(BackupTargetError::FileBackupAlreadyExists.report())
                .attach_printable(name);
        }

        let checksum_file_name = format!("{}.sha256", name);
        let target_checksum_path = BackupDirUtils::new(&config)
            .file_path(&checksum_file_name);
        let target_checksum_file_content = expected_sha256.to_shasum_tool_compatible_checksum(&name);

        Ok(Self {
            expected_packet_number: Wrapping(0),
            target_path,
            target_checksum_path,
            target_checksum_file_content,
            tmp_file_path,
            tmp_file,
            expected_sha256,
            sha256_state: Sha256::new(),
        })
    }

    pub async fn save_packet(
        &mut self,
        packet_number: Wrapping<u32>,
        data: Vec<u8>,
    ) -> Result<(), BackupTargetError> {
        if self.expected_packet_number != packet_number {
            return Err(BackupTargetError::FileBackupPacketNumberMismatch.report())
                .attach_printable(format!("expected: {}, actual: {}", self.expected_packet_number, packet_number))
        }

        self.tmp_file.write_all(&data)
            .await
            .change_context(BackupTargetError::Write)?;

        self.expected_packet_number += 1;

        self.sha256_state.update(&data);

        Ok(())
    }

    pub async fn finalize(
        mut self,
        packet_number: Wrapping<u32>,
    ) -> Result<(), BackupTargetError> {
        if self.expected_packet_number != packet_number {
            return Err(BackupTargetError::FileBackupPacketNumberMismatch.report())
                .attach_printable(format!("expected: {}, actual: {}", self.expected_packet_number, packet_number))
        }

        let received_file_hash = self.sha256_state.finalize();
        if received_file_hash.as_slice() != self.expected_sha256.0 {
            return Err(BackupTargetError::FileBackupDataCorruptionDetected.report())
        }

        tokio::fs::write(self.target_checksum_path, self.target_checksum_file_content)
            .await
            .change_context(BackupTargetError::Write)?;

        self.tmp_file
            .flush()
            .await
            .change_context(BackupTargetError::FileFlush)?;

        self.tmp_file
            .sync_all()
            .await
            .change_context(BackupTargetError::FileSync)?;

        drop(self.tmp_file);

        tokio::fs::rename(self.tmp_file_path, self.target_path)
            .await
            .change_context(BackupTargetError::FileRename)?;

        Ok(())
    }
}

pub struct DeleteOldFileBackups;

impl DeleteOldFileBackups {
    /// Returns how many files were deleted.
    pub async fn run(config: Arc<Config>) -> Result<u64, BackupTargetError> {
        let dir = BackupDirUtils::new(&config).create_files_dir_if_needed();

        let mut iterator = tokio::fs::read_dir(dir)
            .await
            .change_context(BackupTargetError::Read)?;

        let current_time = TryInto::<u64>::try_into(UnixTime::current_time().ut)
            .change_context(BackupTargetError::Time)?;

        let mut deleted_count = 0;

        while let Some(e) = iterator.next_entry().await.change_context(BackupTargetError::Read)? {
            if !e.path().is_file() {
                continue;
            }

            let name = e.file_name();
            let Some(text) = name.to_str() else {
                return Err(BackupTargetError::InvalidFileName.report());
            };

            if !text.starts_with("backup_") {
                continue;
            }

            let unix_time_seconds = e.metadata()
                .await
                .change_context(BackupTargetError::Read)?
                .created()
                .change_context(BackupTargetError::Read)?
                .duration_since(SystemTime::UNIX_EPOCH)
                .change_context(BackupTargetError::Read)?
                .as_secs();

            let deletion_allowed_seconds = unix_time_seconds + Into::<u64>::into(config.backup_link().file_backup_retention_time().seconds);

            if current_time >= deletion_allowed_seconds {
                overwrite_and_remove_if_exists(&e.path())
                    .await
                    .change_context(BackupTargetError::FileOverwritingAndRemovingFailed)?;
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }
}
