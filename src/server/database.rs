
pub mod read;
pub mod write;
pub mod git;
pub mod sqlite;


use std::{
    io, path::{PathBuf, Path}, fs
};

use tokio::{
    sync::{mpsc},
};
use tokio_stream::StreamExt;
use tracing::log::info;

use crate::api::core::user::UserId;

use self::{
    git::{GitError, util::DatabasePath, GitDatabaseOperationHandle}, sqlite::{SqliteDatabasePath, SqliteDatabaseError, SqliteWriteHandle, SqliteWriteCloseHandle, SqliteReadHandle, SqliteReadCloseHandle, read::SqliteReadCommands}, write::WriteCommands, read::ReadCommands,
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
    Derialize(serde_json::Error),
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

        router_handle.check_git_integrity().await?;

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
        let git_dir = self.root.history().user_git_dir(user_id);
        WriteCommands::new(
            git_dir,
            self.git_database_handle.clone(),
            self.sqlite_write.clone()
        )
    }

    pub fn read(&self) -> ReadCommands {
        ReadCommands::new(self.root.history_ref(), &self.sqlite_read)
    }

    /// Make sure that current Git HEAD matches SQLite content.
    /// If not, then do commit with correct files.
    async fn check_git_integrity(&self) -> Result<(), DatabaseError> {
        let read = self.read();
        let mut users = read.sqlite().users();
        while let Some(user_id) = users.try_next().await? {
            let git_write = || self.user_write_commands(&user_id).git_with_mode_message("Integrity check".into());

            // Check that user git repository exists
            let git_dir = self.root.history().user_git_dir(&user_id);
            if !git_dir.exists() {
                git_write().store_user_id().await?;
            }

            // Check profile file
            let git_profile = self.read().git(&user_id).profile().await?;
            let sqlite_profile = read.sqlite().user_profile(&user_id).await?;
            if git_profile.filter(|profile| *profile == sqlite_profile).is_none() {
                git_write().update_user_profile(&sqlite_profile).await?
            }

            // Check ID file
            let git_user_id = self.read().git(&user_id).user_id().await?;
            if git_user_id.filter(|id| *id == user_id).is_none() {
                git_write().update_user_id().await?
            }
        }

        Ok(())
    }
}
