pub mod command;
pub mod file;
pub mod git;
pub mod util;
pub mod sqlite;

use std::{
    io, path::{PathBuf, Path}, fs
};

use tokio::{
    sync::{mpsc},
};

use self::{
    git::{GitError}, util::DatabasePath, sqlite::{SqliteDatabasePath, SqliteDatabaseError, SqliteWriteHandle, SqliteWriteCloseHandle},
};

pub type DatabeseEntryId = String;

/// Every running database write operation should keep this handle. When server
/// quit is started main function waits that all handles are dropped.
#[derive(Debug, Clone)]
pub struct DatabaseOperationHandle {
    _sender: mpsc::Sender<()>,
}

impl DatabaseOperationHandle {
    pub fn new() -> (Self, mpsc::Receiver<()>) {
        let (_sender, receiver) = mpsc::channel(1);
        (Self { _sender }, receiver)
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    Git(GitError),
    FileCreate(io::Error),
    FileOpen(io::Error),
    FileRename(io::Error),
    FileIo(io::Error),
    Serialize(serde_json::Error),
    Init(io::Error),
    Sqlite(SqliteDatabaseError),
}


/// Absolsute path to database root directory.
pub struct DatabaseRoot {
    root: PathBuf,
    history: DatabasePath,
    current: SqliteDatabasePath,
}

impl DatabaseRoot {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, DatabaseError> {
        let root = path.as_ref().to_path_buf();
        if !root.exists() {
            fs::create_dir(&root).map_err(DatabaseError::Init)?;
        }

        let history = root.join("history");
        if !history.exists() {
            fs::create_dir(&history).map_err(DatabaseError::Init)?;
        }
        let history = DatabasePath::new(history);

        let current = root.join("current");
        if !current.exists() {
            fs::create_dir(&current).map_err(DatabaseError::Init)?;
        }
        let current = SqliteDatabasePath::new(current);

        Ok(Self {
            root, history, current
        })
    }

    pub fn history(&self) -> DatabasePath {
        self.history.clone()
    }

    pub fn current(&self) -> SqliteDatabasePath {
        self.current.clone()
    }
}


/// Handle Git and SQLite databases
pub struct DatabaseManager {
    root: DatabaseRoot,
    sqlite_write_close: SqliteWriteCloseHandle,
    sqlite_write: SqliteWriteHandle,
}


impl DatabaseManager {

    /// Runs also some blocking file system code.
    pub async fn new<T: AsRef<Path>>(database_dir: T) -> Result<Self, DatabaseError> {

        let root = DatabaseRoot::new(database_dir)?;

        let (sqlite_write, sqlite_write_close) =
            SqliteWriteHandle::new(root.current()).await.map_err(DatabaseError::Sqlite)?;


        Ok(DatabaseManager {
            root,
            sqlite_write,
            sqlite_write_close,
        })
    }

    pub fn git_path(&self) -> DatabasePath {
        self.root.history()
    }

    pub async fn close(self) {
        self.sqlite_write_close.close().await
    }
}
