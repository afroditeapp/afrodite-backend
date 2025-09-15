use std::{fs, path::PathBuf};

use error_stack::{Result, ResultExt};
use simple_backend_config::{SimpleBackendConfig, SqliteDatabase};

use crate::SimpleDatabaseError;

pub const FILE_DIR_NAME: &str = "files";
pub const SQLITE_DIR_NAME: &str = "sqlite";
pub const SIMPLE_BACKEND_DIR_NAME: &str = "simple_backend";

pub fn create_dirs_and_get_sqlite_database_file_path(
    config: &SimpleBackendConfig,
    database_info: &SqliteDatabase,
) -> Result<PathBuf, SimpleDatabaseError> {
    let sqlite = config.data_dir().join(SQLITE_DIR_NAME);
    if !sqlite.exists() {
        fs::create_dir(&sqlite).change_context(SimpleDatabaseError::FilePathCreationFailed)?;
    }

    let db_dir = sqlite.join(database_info.name);
    if !db_dir.exists() {
        fs::create_dir(&db_dir).change_context(SimpleDatabaseError::FilePathCreationFailed)?;
    }

    let db_file = db_dir.join(format!("{}.db", database_info.name));

    Ok(db_file)
}

pub fn create_dirs_and_get_files_dir_path(
    config: &SimpleBackendConfig,
) -> Result<PathBuf, SimpleDatabaseError> {
    let dir = config.data_dir().join(FILE_DIR_NAME);
    if !dir.exists() {
        fs::create_dir(&dir).change_context(SimpleDatabaseError::FilePathCreationFailed)?;
    }
    Ok(dir)
}

pub fn create_dirs_and_get_simple_backend_dir_path(
    config: &SimpleBackendConfig,
) -> Result<PathBuf, SimpleDatabaseError> {
    let dir = config.data_dir().join(SIMPLE_BACKEND_DIR_NAME);
    if !dir.exists() {
        fs::create_dir(&dir).change_context(SimpleDatabaseError::FilePathCreationFailed)?;
    }

    Ok(dir)
}
