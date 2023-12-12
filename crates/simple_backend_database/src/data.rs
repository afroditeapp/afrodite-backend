use std::{fs, path::PathBuf};

use error_stack::{Result, ResultExt};
// use model::{AccountId, AccountIdInternal, IsLoggingAllowed, SignInWithInfo};
use simple_backend_config::{file::SqliteDatabase, SimpleBackendConfig};

use crate::DataError;

pub const FILE_DIR_NAME: &str = "files";
pub const SQLITE_DIR_NAME: &str = "sqlite";

pub fn create_dirs_and_get_sqlite_database_file_path(
    config: &SimpleBackendConfig,
    database_info: &SqliteDatabase,
) -> Result<PathBuf, DataError> {
    let root = config.data_dir().to_path_buf();
    if !root.exists() {
        fs::create_dir(&root).change_context(DataError::FilePathCreationFailed)?;
    }

    let sqlite = root.join(SQLITE_DIR_NAME);
    if !sqlite.exists() {
        fs::create_dir(&sqlite).change_context(DataError::FilePathCreationFailed)?;
    }

    let db_dir = root.join(database_info.name.clone());
    if !db_dir.exists() {
        fs::create_dir(&db_dir).change_context(DataError::FilePathCreationFailed)?;
    }

    let db_file = root.join(format!("{}.db", database_info.name));

    Ok(db_file)
}

pub fn create_dirs_and_get_files_dir_path(
    config: &SimpleBackendConfig,
) -> Result<PathBuf, DataError> {
    let root = config.data_dir().to_path_buf();
    if !root.exists() {
        fs::create_dir(&root).change_context(DataError::FilePathCreationFailed)?;
    }

    let dir = root.join(FILE_DIR_NAME);
    if !dir.exists() {
        fs::create_dir(&dir).change_context(DataError::FilePathCreationFailed)?;
    }

    Ok(dir)
}

// /// Absolsute path to database root directory.
// #[derive(Clone, Debug)]
// pub struct DatabaseRoot {
//     root: PathBuf,
//     history: SqliteDatabasePath,
//     current: SqliteDatabasePath,
//     file_dir: FileDir,
// }

// impl DatabaseRoot {
//     pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, SqliteDatabaseError> {
//         let root = path.as_ref().to_path_buf();
//         if !root.exists() {
//             fs::create_dir(&root).change_context(SqliteDatabaseError::Init)?;
//         }

//         let history = root.join(DB_HISTORY_DIR_NAME);
//         if !history.exists() {
//             fs::create_dir(&history).change_context(SqliteDatabaseError::Init)?;
//         }
//         let history = SqliteDatabasePath::new(history);

//         let current = root.join(DB_CURRENT_DATA_DIR_NAME);
//         if !current.exists() {
//             fs::create_dir(&current).change_context(SqliteDatabaseError::Init)?;
//         }
//         let current = SqliteDatabasePath::new(current);

//         let file_dir = root.join(DB_FILE_DIR_NAME);
//         if !file_dir.exists() {
//             fs::create_dir(&file_dir).change_context(SqliteDatabaseError::Init)?;
//         }
//         let file_dir = FileDir::new(file_dir);

//         Ok(Self {
//             root,
//             history,
//             current,
//             file_dir,
//         })
//     }

//     /// History Sqlite database path
//     pub fn history(&self) -> SqliteDatabasePath {
//         self.history.clone()
//     }

//     pub fn history_ref(&self) -> &SqliteDatabasePath {
//         &self.history
//     }

//     /// Sqlite database path
//     pub fn current(&self) -> SqliteDatabasePath {
//         self.current.clone()
//     }

//     pub fn current_ref(&self) -> &SqliteDatabasePath {
//         &self.current
//     }

//     pub fn file_dir(&self) -> &FileDir {
//         &self.file_dir
//     }

//     pub fn current_db_file(&self) -> PathBuf {
//         self.current
//             .clone()
//             .path()
//             .join(DatabaseType::Current.to_file_name())
//     }

//     pub fn history_db_file(&self) -> PathBuf {
//         self.history
//             .clone()
//             .path()
//             .join(DatabaseType::History.to_file_name())
//     }
// }
