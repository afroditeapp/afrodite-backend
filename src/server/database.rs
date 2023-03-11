pub mod git;
pub mod read;
pub mod sqlite;
pub mod write;
pub mod utils;

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use error_stack::{Result, ResultExt};

use crate::{
    api::model::{AccountId, Profile, Account, AccountSetup},
    utils::{AppendErr, ErrorContainer, ErrorConversion},
};

use self::{
    git::{utils::DatabasePath, GitDatabaseOperationHandle, GitError, write::GitDatabaseWriteCommands, file::GitJsonFile},
    read::{ReadCmd, ReadCommands},
    sqlite::{
        SqliteDatabasePath, SqliteReadCloseHandle, SqliteReadHandle, SqliteWriteCloseHandle,
        SqliteWriteHandle, utils::SqliteSelectJson,
    },
    write::{WriteCmd, WriteCommands}, utils::GetReadWriteCmd,
};
use crate::utils::IntoReportExt;

pub type DatabeseEntryId = String;

#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Git error")]
    Git,
    #[error("SQLite error")]
    Sqlite,

    // Other errors
    #[error("Database initialization error")]
    Init,
    #[error("Database SQLite and Git integrity check")]
    Integrity,
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
            fs::create_dir(&root).into_error(DatabaseError::Init)?;
        }

        let history = root.join("history");
        if !history.exists() {
            fs::create_dir(&history).into_error(DatabaseError::Init)?;
        }
        let history = DatabasePath::new(history);

        let current = root.join("current");
        if !current.exists() {
            fs::create_dir(&current).into_error(DatabaseError::Init)?;
        }
        let current = SqliteDatabasePath::new(current);

        Ok(Self {
            root,
            history,
            current,
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
    pub async fn new<T: AsRef<Path>>(
        database_dir: T,
    ) -> Result<(Self, RouterDatabaseHandle), DatabaseError> {
        let root = DatabaseRoot::new(database_dir)?;

        let (sqlite_write, sqlite_write_close) = SqliteWriteHandle::new(root.current())
            .await
            .change_context(DatabaseError::Init)?;

        let (sqlite_read, sqlite_read_close) = SqliteReadHandle::new(root.current())
            .await
            .change_context(DatabaseError::Init)?;

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

    pub fn user_write_commands(&self, user_id: &AccountId) -> WriteCommands {
        let git_dir = self.root.history().user_git_dir(user_id);
        WriteCommands::new(
            git_dir,
            self.git_database_handle.clone(),
            self.sqlite_write.clone(),
        )
    }

    pub fn read(&self) -> ReadCommands {
        ReadCommands::new(self.root.history_ref(), &self.sqlite_read)
    }

    /// Make sure that current Git HEAD matches SQLite content.
    /// If not, then do commit with correct files.
    async fn check_git_integrity(&self) -> Result<(), DatabaseError> {
        let read = self.read();
        let mut users = read.sqlite().account_ids();
        let mut error: ErrorContainer<DatabaseError> = None;

        while let Some(result) = users
            .next()
            .await
            .map(|r| r.change_context(DatabaseError::Integrity))
        {
            match result {
                Ok(user_id) => {
                    let result = self
                        .integrity_check_handle_user_id(user_id, &read)
                        .await
                        .map_err(|r| r.change_context(DatabaseError::Integrity));
                    match result {
                        Ok(()) => (),
                        Err(e) => error.append(e),
                    }
                }
                Err(e) => error.append(e),
            }
        }

        error.into_result()

        // TODO: Just stop integrity check if one error occurs?
    }

    async fn integrity_check_handle_user_id(
        &self,
        id: AccountId,
        read: &ReadCommands<'_>,
    ) -> Result<(), DatabaseError> {
        let git_write = || {
            self.user_write_commands(&id)
                .git_with_mode_message("Integrity check".into())
        };

        // Check that user git repository exists
        let git_dir = self.root.history().user_git_dir(&id);
        if !git_dir.exists() {
            git_write()
                .store_account_id()
                .await
                .with_info_lazy(|| WriteCmd::AccountId(id.clone()))?;
        }

        Self::check_integrity_json::<Profile>(&id, read, git_write).await?;
        Self::check_integrity_json::<Account>(&id, read, git_write).await?;
        Self::check_integrity_json::<AccountSetup>(&id, read, git_write).await?;

        // Check ID file
        let git_id = self
            .read()
            .git(&id)
            .account_id()
            .await
            .with_info_lazy(|| ReadCmdIntegrity::AccountId(id.clone()))?;
        if git_id.filter(|git_id| *git_id == id).is_none() {
            git_write()
                .update_user_id()
                .await
                .with_info_lazy(|| WriteCmdIntegrity::GitAccountIdFile(id.clone()))?;
        }

        Ok(())
    }

    async fn check_integrity_json<
        T: DeserializeOwned + GitJsonFile + PartialEq + GetReadWriteCmd + SqliteSelectJson + Send + Clone + 'static
    >(
        id: &AccountId,
        read: &ReadCommands<'_>,
        write: impl Fn() -> GitDatabaseWriteCommands,
    ) -> Result<(), DatabaseError> {
        let git_data: Option<T> = read
            .git(id)
            .read_json()
            .await
            .with_read_cmd_info::<T>(id)?;
        let sqlite_data = T::select_json(id, read.sqlite())
            .await
            .with_read_cmd_info::<T>(id)?;
        if git_data
            .filter(|data| *data == sqlite_data)
            .is_none() {
                write()
                    .update_json(&sqlite_data)
                    .await
                    .with_write_cmd_info::<T>(id)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
enum WriteCmdIntegrity {
    GitAccountIdFile(AccountId),
}

impl std::fmt::Display for WriteCmdIntegrity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Integrity write command: {:?}", self))
    }
}

#[derive(Debug, Clone)]
enum ReadCmdIntegrity {
    AccountId(AccountId),
}

impl std::fmt::Display for ReadCmdIntegrity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Read command: {:?}", self))
    }
}
