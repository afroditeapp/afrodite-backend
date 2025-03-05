use std::path::{Path, PathBuf};

use axum::body::BodyDataStream;
use error_stack::{Result, ResultExt};
use model::{AccountId, ContentId, ContentProcessingId};
use simple_backend_utils::{file::overwrite_and_remove_if_exists, ContextExt};
use tokio::io::AsyncWriteExt;
use tokio_stream::{wrappers::ReadDirStream, StreamExt};
use tokio_util::io::ReaderStream;

use super::super::FileError;

pub const TMP_DIR_NAME: &str = "tmp";
pub const CONTENT_DIR_NAME: &str = "content";
pub const EXPORT_DIR_NAME: &str = "export";

const MAX_TMP_FILE_SIZE: usize = 1024 * 1024 * 1024 * 10; // 10 MiB

/// Path to directory which contains all account data directories.
#[derive(Debug, Clone)]
pub struct FileDir {
    dir: PathBuf,
}

impl FileDir {
    pub fn new<T: AsRef<Path>>(file_dir: T) -> Self {
        Self {
            dir: file_dir.as_ref().to_path_buf(),
        }
    }

    /// Unprocessed content upload.
    pub fn raw_content_upload(&self, id: AccountId, content_id: ContentProcessingId) -> TmpContentFile {
        self.account_dir(id).tmp_dir().raw_content_upload(content_id)
    }

    pub fn processed_content_upload(&self, id: AccountId, content_id: ContentProcessingId) -> TmpContentFile {
        self.account_dir(id)
            .tmp_dir()
            .processed_content_upload(content_id)
    }

    pub fn media_content(&self, id: AccountId, content_id: ContentId) -> ContentFile {
        self.account_dir(id).content_dir().media_content(content_id)
    }

    pub fn account_dir(&self, id: AccountId) -> AccountDir {
        let mut dir = self.dir.clone();
        dir.push(id.to_string());
        AccountDir { dir }
    }

    pub fn tmp_dir(&self, id: AccountId) -> TmpDir {
        self.account_dir(id).tmp_dir()
    }

    pub fn path(&self) -> &Path {
        &self.dir
    }
}

#[derive(Debug, Clone)]
pub struct AccountDir {
    dir: PathBuf,
}

impl AccountDir {
    fn tmp_dir(mut self) -> TmpDir {
        self.dir.push(TMP_DIR_NAME);
        TmpDir { dir: self.dir }
    }

    // TODO(prod): Remove if not used
    fn _export_dir(mut self) -> ExportDir {
        self.dir.push(EXPORT_DIR_NAME);
        ExportDir { dir: self.dir }
    }

    fn content_dir(mut self) -> ContentDir {
        self.dir.push(CONTENT_DIR_NAME);
        ContentDir { dir: self.dir }
    }

    pub async fn delete_if_exists(self) -> Result<(), FileError> {
        if self.dir.exists() {
            tokio::fs::remove_dir_all(self.dir)
                .await
                .change_context(FileError::IoFileRemove)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct TmpDir {
    dir: PathBuf,
}

impl TmpDir {
    pub fn path(&self) -> &PathBuf {
        &self.dir
    }

    /// Remove dir contents
    ///
    /// Does not do anything if dir does not exists.
    pub async fn overwrite_and_remove_contents_if_exists(&self) -> Result<(), FileError> {
        if !self.dir.exists() {
            return Ok(());
        }

        if self
            .dir
            .file_name()
            .ok_or(FileError::MissingFileName)?
            .to_string_lossy()
            == TMP_DIR_NAME
        {
            let iter = tokio::fs::read_dir(&self.dir)
                .await
                .change_context(FileError::IoDirIter)?;
            let mut s = ReadDirStream::new(iter);
            while let Some(entry) = s.next().await {
                let entry = entry.change_context(FileError::IoDirIter)?;
                let path = PathToFile {
                    path: entry.path(),
                };
                path.overwrite_and_remove_if_exists().await?;
            }
            Ok(())
        } else {
            Err(FileError::InvalidDirectory.report())
        }
    }

    pub fn raw_content_upload(mut self, id: ContentProcessingId) -> TmpContentFile {
        self.dir.push(id.raw_content_file_name());
        TmpContentFile {
            path: PathToFile { path: self.dir },
        }
    }

    pub fn processed_content_upload(mut self, id: ContentProcessingId) -> TmpContentFile {
        self.dir.push(id.content_file_name());
        TmpContentFile {
            path: PathToFile { path: self.dir },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContentDir {
    dir: PathBuf,
}

impl ContentDir {
    pub fn path(&self) -> &PathBuf {
        &self.dir
    }

    pub fn media_content(mut self, content_id: ContentId) -> ContentFile {
        self.dir.push(content_id.content_file_name());
        ContentFile {
            path: PathToFile { path: self.dir },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportDir {
    dir: PathBuf,
}

impl ExportDir {
    pub fn path(&self) -> &PathBuf {
        &self.dir
    }
}

#[derive(Debug, Clone)]
pub struct ContentFile {
    path: PathToFile,
}

impl ContentFile {
    pub fn path(&self) -> &PathBuf {
        self.path.as_path()
    }

    pub async fn overwrite_and_remove_if_exists(self) -> Result<(), FileError> {
        self.path.overwrite_and_remove_if_exists().await
    }

    pub async fn byte_count_and_read_stream(
        &self,
    ) -> Result<(u64, ReaderStream<tokio::fs::File>), FileError> {
        self.path.byte_count_and_read_stream().await
    }

    pub async fn read_all(&self) -> Result<Vec<u8>, FileError> {
        self.path.read_all().await
    }
}

#[derive(Debug, Clone)]
pub struct TmpContentFile {
    path: PathToFile,
}

impl TmpContentFile {
    pub async fn save_stream(&self, stream: BodyDataStream) -> Result<(), FileError> {
        self.path.save_stream(stream).await
    }

    pub async fn move_to(self, new_location: &ContentFile) -> Result<(), FileError> {
        self.path.move_to(&new_location.path).await
    }

    pub fn move_to_blocking(self, new_location: &ContentFile) -> Result<(), FileError> {
        self.path.move_to_blocking(&new_location.path)
    }

    pub async fn overwrite_and_remove_if_exists(self) -> Result<(), FileError> {
        self.path.overwrite_and_remove_if_exists().await
    }

    pub fn as_path(&self) -> &Path {
        self.path.as_path()
    }
}

#[derive(Debug, Clone)]
struct PathToFile {
    path: PathBuf,
}

impl PathToFile {
    pub fn as_path(&self) -> &PathBuf {
        &self.path
    }

    pub async fn create_parent_dirs(&self) -> Result<(), FileError> {
        if let Some(parent_dir) = self.path.parent() {
            if !parent_dir.exists() {
                tokio::fs::create_dir_all(parent_dir)
                    .await
                    .change_context(FileError::IoFileCreate)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub fn create_parent_dirs_blocking(&self) -> Result<(), FileError> {
        if let Some(parent_dir) = self.path.parent() {
            if !parent_dir.exists() {
                std::fs::create_dir_all(parent_dir).change_context(FileError::IoFileCreate)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub async fn save_stream(&self, mut stream: BodyDataStream) -> Result<(), FileError> {
        self.create_parent_dirs().await?;

        let mut file = tokio::fs::File::create(&self.path)
            .await
            .change_context(FileError::IoFileCreate)?;

        let mut file_size = 0;

        while let Some(result) = stream.next().await {
            let mut data = result.change_context(FileError::StreamReadFailed)?;
            file_size += data.len();
            if file_size > MAX_TMP_FILE_SIZE {
                return Err(FileError::FileUploadMaxFileSizeReached.report());
            }
            file.write_all_buf(&mut data)
                .await
                .change_context(FileError::IoFileWrite)?;
        }
        file.flush().await.change_context(FileError::IoFileFlush)?;
        file.sync_all()
            .await
            .change_context(FileError::IoFileSync)?;
        Ok(())
    }

    pub async fn byte_count_and_read_stream(
        &self,
    ) -> Result<(u64, ReaderStream<tokio::fs::File>), FileError> {
        let file = tokio::fs::File::open(&self.path)
            .await
            .change_context(FileError::IoFileOpen)?;
        let metadata = file
            .metadata()
            .await
            .change_context(FileError::IoFileMetadata)?;
        Ok((metadata.len(), ReaderStream::new(file)))
    }

    pub async fn read_all(&self) -> Result<Vec<u8>, FileError> {
        tokio::fs::read(&self.path)
            .await
            .change_context(FileError::IoFileOpen)
    }

    pub async fn move_to(self, new_location: &Self) -> Result<(), FileError> {
        new_location.create_parent_dirs().await?;

        tokio::fs::rename(self.path, new_location.as_path())
            .await
            .change_context(FileError::IoFileRename)
    }

    pub fn move_to_blocking(self, new_location: &Self) -> Result<(), FileError> {
        new_location.create_parent_dirs_blocking()?;

        std::fs::rename(self.path, new_location.as_path()).change_context(FileError::IoFileRename)
    }

    pub async fn overwrite_and_remove_if_exists(self) -> Result<(), FileError> {
        overwrite_and_remove_if_exists(&self.path)
            .await
            .change_context(FileError::FileOverwritingAndRemovingFailed)
    }
}
