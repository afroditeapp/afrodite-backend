use std::{
    fs,
    path::{Path, PathBuf},
};

use error_stack::Result;

use crate::api::model::{AccountId, AccountIdLight};

use super::{
    super::FileError,
    file::{GetStaticFileName, ImageSlot},
    read::FileReadCommands,
};

use crate::utils::IntoReportExt;

pub const SLOT_DIR_NAME: &str = "slot";
pub const IMAGE_DIR_NAME: &str = "image";
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

    pub fn slot(&self, id: &AccountIdLight, slot: ImageSlot) -> PathToFile {
        let mut dir = self.dir.clone();
        dir.push(id.to_string());
        dir.push(SLOT_DIR_NAME);
        dir.push(slot.file_name().as_str());
        PathToFile { path: dir }
    }

    pub fn account_dir(&self, id: &AccountIdLight) -> AccountDir {
        let mut dir = self.dir.clone();
        dir.push(id.to_string());
        AccountDir {
            dir,
        }
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
    pub fn path(&self) -> &PathBuf {
        &self.dir
    }

    pub fn slot_dir(mut self, ) -> SlotDir {
        self.dir.push(SLOT_DIR_NAME);
        SlotDir {
            dir: self.dir,
        }
    }

    pub fn export_dir(mut self) -> ExportDir {
        self.dir.push(EXPORT_DIR_NAME);
        ExportDir {
            dir: self.dir,
        }
    }

    pub fn image_dir(mut self) -> ImageDir {
        self.dir.push(IMAGE_DIR_NAME);
        ImageDir {
            dir: self.dir,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SlotDir {
    dir: PathBuf,
}


impl SlotDir {
    pub fn path(&self) -> &PathBuf {
        &self.dir
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
pub struct SlotFile {
    path: PathBuf,
}


impl SlotFile {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[derive(Debug, Clone)]
pub struct ImageFile {
    path: PathBuf,
}


impl ImageFile {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[derive(Debug, Clone)]
pub struct PathToFile {
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
}

/*


    pub async fn read_to_string_optional<T: GetLiveVersionPath>(
        &self,
        file: T,
    ) -> Result<Option<String>, GitError> {
        let path = self.account_dir.join(file.live_path().as_str());
        if !path.is_file() {
            return Ok(None);
        }
        tokio::fs::read_to_string(path)
            .await
            .into_error_with_info(GitError::IoFileRead, file.live_path())
            .map(Some)
    }

    pub async fn read_to_string<T: GetLiveVersionPath>(&self, file: T) -> Result<String, GitError> {
        let path = self.account_dir.join(file.live_path().as_str());
        tokio::fs::read_to_string(path)
            .await
            .into_error_with_info(GitError::IoFileRead, file.live_path())
    }

    /// Open file for reading.
    pub fn open_file<T: GetLiveVersionPath>(&self, file: T) -> Result<fs::File, GitError> {
        let path = self.account_dir.join(file.live_path().as_str());
        fs::File::open(path).into_error_with_info(GitError::IoFileOpen, file.live_path())
    }

    /// Replace file using new file. Creates the file if it does not exists.
    pub fn replace_file<
        T: GetStaticFileName + GetLiveVersionPath + Copy,
        U: FnMut(&mut fs::File) -> Result<(), GitError>,
    >(
        &self,
        file: T,
        commit_msg: &str,
        mut write_handle: U,
    ) -> Result<(), GitError> {
        let git_file_path = self.account_dir.join(file.git_path().as_str());
        let mut git_file = fs::File::create(&git_file_path)
            .into_error_with_info_lazy(GitError::IoFileCreate, || {
                git_file_path.clone().to_string_lossy().to_string()
            })?;

        write_handle(&mut git_file)?;
        drop(git_file);

        let _git = GitDatabase::open(self)?;
        let _msg = match self.mode_msg.as_ref() {
            Some(mode_msg) => format!("{}\n\n{}", mode_msg, commit_msg),
            None => commit_msg.to_owned(),
        };

        let live_file_path = self.account_dir.join(file.live_path().as_str());
        fs::rename(&git_file_path, &live_file_path).into_error_with_info_lazy(
            GitError::IoFileRename,
            || {
                format!(
                    "from: {} to: {}",
                    git_file_path.to_string_lossy(),
                    live_file_path.to_string_lossy(),
                )
            },
        )
    }

    pub fn replace_no_history_file<
        T: GetTmpPath + GetLiveVersionPath + Copy,
        U: FnMut(&mut fs::File) -> Result<(), GitError>,
    >(
        &self,
        file: T,
        mut write_handle: U,
    ) -> Result<(), GitError> {
        let tmp_file_path = self.account_dir.join(file.tmp_path().as_str());
        let mut tmp_file = fs::File::create(&tmp_file_path)
            .into_error_with_info(GitError::IoFileCreate, file.tmp_path())?;

        write_handle(&mut tmp_file)?;
        drop(tmp_file);

        let live_file_path = self.account_dir.join(file.live_path().as_str());
        fs::rename(&tmp_file_path, &live_file_path).into_error_with_info_lazy(
            GitError::IoFileRename,
            || {
                format!(
                    "from: {} to: {}",
                    tmp_file_path.to_string_lossy(),
                    live_file_path.to_string_lossy(),
                )
            },
        )
    }

    pub fn read(&self) -> GitDatabaseReadCommands {
        GitDatabaseReadCommands::new(self.clone())
    }

 */
