
pub mod read;
pub mod write;
pub mod git;
pub mod sqlite;


use std::{
    io, path::{PathBuf, Path}, fs, f64::consts::E
};

use tokio::{
    sync::{mpsc},
};

use crate::api::core::user::UserId;

use self::{
    git::{GitError, util::DatabasePath, GitDatabaseOperationHandle}, sqlite::{SqliteDatabasePath, SqliteDatabaseError, SqliteWriteHandle, SqliteWriteCloseHandle, SqliteReadHandle, SqliteReadCloseHandle}, write::WriteCommands, read::ReadCommands,
};

pub type DatabeseEntryId = String;



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
    FileSystem(io::Error),
    Utf8,
}

impl From<GitError> for DatabaseError {
    fn from(e: GitError) -> Self {
        DatabaseError::Git(e)
    }
}

impl From<SqliteDatabaseError> for DatabaseError {
    fn from(e: SqliteDatabaseError) -> Self {
        DatabaseError::Sqlite(e)
    }
}

impl From<std::io::Error> for DatabaseError {
    fn from(e: std::io::Error) -> Self {
        DatabaseError::FileSystem(e)
    }
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

    /// Directory containing user git repositories
    pub fn history(&self) -> DatabasePath {
        self.history.clone()
    }

    pub fn history_ref(&self) -> &DatabasePath {
        &self.history
    }

    /// Sqlite database path
    pub fn current(&self) -> SqliteDatabasePath {
        self.current.clone()
    }

    pub fn current_ref(&self) -> &SqliteDatabasePath {
        &self.current
    }
}


/// Handle Git and SQLite databases
pub struct DatabaseManager {
    sqlite_write_close: SqliteWriteCloseHandle,
    sqlite_read_close: SqliteReadCloseHandle,
    git_quit_receiver: mpsc::Receiver<()>,
}


impl DatabaseManager {

    /// Runs also some blocking file system code.
    pub async fn new<T: AsRef<Path>>(database_dir: T) -> Result<(Self, RouterDatabaseHandle), DatabaseError> {

        let root = DatabaseRoot::new(database_dir)?;

        let (sqlite_write, sqlite_write_close) =
            SqliteWriteHandle::new(root.current()).await.map_err(DatabaseError::Sqlite)?;

        let (sqlite_read, sqlite_read_close) =
            SqliteReadHandle::new(root.current()).await.map_err(DatabaseError::Sqlite)?;

        let (git_database_handle, git_quit_receiver) = GitDatabaseOperationHandle::new();


        // TODO: Check that git directories current state matches with the
        //       sqlite database.

        let database_manager = DatabaseManager {
            sqlite_write_close,
            sqlite_read_close,
            git_quit_receiver,
        };

        let router_handle = RouterDatabaseHandle {
            sqlite_write,
            sqlite_read,
            root,
            git_database_handle,
        };

        Ok((database_manager, router_handle))
    }

    pub async fn close(mut self) {
        self.sqlite_read_close.close().await;
        self.sqlite_write_close.close().await;
        loop {
            match self.git_quit_receiver.recv().await {
                None => break,
                Some(()) => (),
            }
        }
    }
}


pub struct RouterDatabaseHandle {
    root: DatabaseRoot,
    sqlite_write: SqliteWriteHandle,
    sqlite_read: SqliteReadHandle,
    git_database_handle: GitDatabaseOperationHandle,
}

impl RouterDatabaseHandle {
    pub fn git_path(&self) -> DatabasePath {
        self.root.history()
    }


    pub fn user_write_commands(&self, user_id: &UserId) -> WriteCommands {
        let git_dir = self.root.history().user_git_dir(&user_id);
        WriteCommands::new(
            git_dir,
            self.git_database_handle.clone(),
            self.sqlite_write.clone()
        )
    }

    pub fn read(&self) -> ReadCommands {
        ReadCommands::new(self.root.history_ref(), &self.sqlite_read)
    }
}
