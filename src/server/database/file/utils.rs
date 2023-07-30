use std::path::{Path, PathBuf};

use axum::extract::BodyStream;
use error_stack::Result;
use tokio::io::AsyncWriteExt;
use tokio_stream::{wrappers::ReadDirStream, StreamExt};
use tokio_util::io::ReaderStream;

use crate::{
    api::model::{AccountIdLight, ContentId},
    server::database::read::ReadResult,
};

use super::{super::FileError, file::GetStaticFileName};

use crate::utils::IntoReportExt;

pub const TMP_DIR_NAME: &str = "tmp";
pub const IMAGE_DIR_NAME: &str = "images";
pub const EXPORT_DIR_NAME: &str = "export";

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

    pub fn unprocessed_image_upload(&self, id: AccountIdLight, content: ContentId) -> TmpImageFile {
        self.account_dir(id)
            .tmp_dir()
            .unprocessed_image_upload(content)
    }

    pub fn image_content(&self, id: AccountIdLight, content_id: ContentId) -> ImageFile {
        self.account_dir(id).image_dir().image_content(content_id)
    }

    pub fn account_dir(&self, id: AccountIdLight) -> AccountDir {
        let mut dir = self.dir.clone();
        dir.push(id.to_string());
        AccountDir { dir }
    }

    pub fn tmp_dir(&self, id: AccountIdLight) -> TmpDir {
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
    fn path(&self) -> &PathBuf {
        &self.dir
    }

    fn tmp_dir(mut self) -> TmpDir {
        self.dir.push(TMP_DIR_NAME);
        TmpDir { dir: self.dir }
    }

    fn export_dir(mut self) -> ExportDir {
        self.dir.push(EXPORT_DIR_NAME);
        ExportDir { dir: self.dir }
    }

    fn image_dir(mut self) -> ImageDir {
        self.dir.push(IMAGE_DIR_NAME);
        ImageDir { dir: self.dir }
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
    pub async fn remove_contents_if_exists(&self) -> Result<(), FileError> {
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
                .into_error(FileError::IoDirIter)?;
            let mut s = ReadDirStream::new(iter);
            while let Some(entry) = s.next().await {
                let entry = entry.into_error(FileError::IoDirIter)?;
                tokio::fs::remove_file(entry.path())
                    .await
                    .into_error(FileError::IoFileRemove)?;
            }
            Ok(())
        } else {
            Err(FileError::InvalidDirectory.into())
        }
    }

    pub fn unprocessed_image_upload(mut self, id: ContentId) -> TmpImageFile {
        self.dir.push(id.raw_jpg_image());
        TmpImageFile {
            path: PathToFile { path: self.dir },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageDir {
    dir: PathBuf,
}

impl ImageDir {
    pub fn path(&self) -> &PathBuf {
        &self.dir
    }

    pub fn image_content(mut self, content_id: ContentId) -> ImageFile {
        self.dir.push(content_id.jpg_image());
        ImageFile {
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
pub struct ImageFile {
    path: PathToFile,
}

impl ImageFile {
    pub fn path(&self) -> &PathBuf {
        self.path.path()
    }

    pub async fn remove_if_exists(self) -> Result<(), FileError> {
        self.path.remove_if_exists().await
    }

    pub async fn read_stream(
        &self,
    ) -> ReadResult<ReaderStream<tokio::fs::File>, FileError, ImageFile> {
        self.path.read_stream().await.map_err(|e| e.into())
    }

    pub async fn read_all(&self) -> ReadResult<Vec<u8>, FileError, ImageFile> {
        self.path.read_all().await.map_err(|e| e.into())
    }
}

#[derive(Debug, Clone)]
pub struct TmpImageFile {
    path: PathToFile,
}

impl TmpImageFile {
    pub async fn save_stream(&self, stream: BodyStream) -> Result<(), FileError> {
        self.path.save_stream(stream).await
    }

    pub async fn move_to(self, new_location: &ImageFile) -> Result<(), FileError> {
        self.path.move_to(&new_location.path).await
    }

    pub async fn remove_if_exists(self) -> Result<(), FileError> {
        self.path.remove_if_exists().await
    }
}

#[derive(Debug, Clone)]
struct PathToFile {
    path: PathBuf,
}

impl PathToFile {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub async fn create_parent_dirs(&self) -> Result<(), FileError> {
        if let Some(parent_dir) = self.path.parent() {
            if !parent_dir.exists() {
                tokio::fs::create_dir_all(parent_dir)
                    .await
                    .into_error(FileError::IoFileCreate)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub async fn save_stream(&self, mut stream: BodyStream) -> Result<(), FileError> {
        self.create_parent_dirs().await?;

        let mut file = tokio::fs::File::create(&self.path)
            .await
            .into_error(FileError::IoFileCreate)?;

        while let Some(result) = stream.next().await {
            let mut data = result.into_error(FileError::StreamReadFailed)?;
            file.write_all_buf(&mut data)
                .await
                .into_error(FileError::IoFileWrite)?;
        }
        file.flush().await.into_error(FileError::IoFileFlush)?;
        file.sync_all().await.into_error(FileError::IoFileSync)?;
        Ok(())
    }

    pub async fn read_stream(&self) -> Result<ReaderStream<tokio::fs::File>, FileError> {
        let file = tokio::fs::File::open(&self.path)
            .await
            .into_error(FileError::IoFileOpen)?;
        Ok(ReaderStream::new(file))
    }

    pub async fn read_all(&self) -> Result<Vec<u8>, FileError> {
        tokio::fs::read(&self.path)
            .await
            .into_error(FileError::IoFileOpen)
    }

    pub async fn move_to(self, new_location: &Self) -> Result<(), FileError> {
        new_location.create_parent_dirs().await?;

        tokio::fs::rename(self.path, new_location.path())
            .await
            .into_error(FileError::IoFileRename)
    }

    pub async fn remove_if_exists(self) -> Result<(), FileError> {
        if !self.exists() {
            return Ok(());
        }

        tokio::fs::remove_file(&self.path)
            .await
            .into_error(FileError::IoFileRemove)
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}
