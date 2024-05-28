use std::{
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use config::Config;
use database::{
    CurrentReadHandle, CurrentWriteHandle, ErrorContext,
    HistoryReadHandle, HistoryWriteHandle,
};
use error_stack::Context;
use model::{AccountId, AccountIdInternal, EmailAddress, IsLoggingAllowed, SignInWithInfo};
use simple_backend::media_backup::MediaBackupHandle;
use simple_backend_database::{DatabaseHandleCreator, DbReadCloseHandle, DbWriteCloseHandle};
use tracing::info;

use self::{
    cache::{CacheError, DatabaseCache},
    file::{read::FileReadCommands, utils::FileDir, FileError},
    index::{LocationIndexIteratorHandle, LocationIndexManager},
    read::ReadCommands,
    utils::{AccessTokenManager, AccountIdManager},
    write::{
        account::WriteCommandsAccount, account_admin::WriteCommandsAccountAdmin,
        chat::WriteCommandsChat, chat_admin::WriteCommandsChatAdmin, common::WriteCommandsCommon,
        media::WriteCommandsMedia, media_admin::WriteCommandsMediaAdmin,
        profile::WriteCommandsProfile, profile_admin::WriteCommandsProfileAdmin, WriteCommands,
    },
    write_concurrent::WriteCommandsConcurrent,
};
use crate::{
    event::EventManagerWithCacheReference, internal_api::InternalApiError, push_notifications::PushNotificationSender, result::{Result, WrappedReport}
};

pub mod cache;
pub mod file;
pub mod index;
pub mod read;
pub mod utils;
pub mod write;
pub mod write_commands;
pub mod write_concurrent;

pub const DB_FILE_DIR_NAME: &str = "files";

pub type DatabeseEntryId = String;

#[derive(thiserror::Error, Debug)]
pub enum DataError {
    #[error("Git error")]
    Git,
    #[error("SQLite error")]
    Sqlite,
    #[error("Cache error")]
    Cache,
    #[error("File error")]
    File,
    #[error("I/O error")]
    Io,
    #[error("Profile index error")]
    ProfileIndex,
    #[error("Media backup error")]
    MediaBackup,
    #[error("Image process error")]
    ImageProcess,

    #[error("Diesel error")]
    Diesel,

    #[error("Database command sending failed")]
    CommandSendingFailed,
    #[error("Database command result receiving failed")]
    CommandResultReceivingFailed,

    // Other errors
    #[error("Database initialization error")]
    Init,
    #[error("Database SQLite and Git integrity check")]
    Integrity,
    #[error("Feature disabled from config file")]
    FeatureDisabled,
    #[error("Not found")]
    NotFound,
    #[error("Tried to do something that is not allowed")]
    NotAllowed,
    #[error("Action is already done")]
    AlreadyDone,
    #[error("Server closing in progress")]
    ServerClosingInProgress,

    #[error("Command runner quit too early")]
    CommandRunnerQuit,

    #[error("Event mode access failed")]
    EventModeAccessFailed,
}

/// Attach more info to current error
///
/// This trait is for error container error_stack::Report<Err>
pub trait WithInfo<Ok, Err: Context>: Sized {
    fn into_error_without_context(self) -> std::result::Result<Ok, error_stack::Report<Err>>;

    #[track_caller]
    fn with_info<T: Debug + IsLoggingAllowed>(
        self,
        request_context: T,
    ) -> std::result::Result<Ok, error_stack::Report<Err>> {
        self.into_error_without_context().map_err(|e| {
            e.attach_printable(ErrorContext::<T, Ok>::new(request_context).printable())
        })
    }
}

impl<Ok, Err: Context> WithInfo<Ok, Err> for std::result::Result<Ok, error_stack::Report<Err>> {
    #[track_caller]
    fn into_error_without_context(self) -> std::result::Result<Ok, error_stack::Report<Err>> {
        self
    }
}

/// Attach more info to current error.
///
/// This trait is for error container WrappedReport<error_stack::Report<Err>>
pub trait WrappedWithInfo<Ok, Err: Context>: Sized {
    fn into_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>>;

    #[track_caller]
    fn with_info<T: Debug + IsLoggingAllowed>(
        self,
        request_context: T,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>> {
        self.into_error_without_context().map_err(|e| {
            e.attach_printable(ErrorContext::<T, Ok>::new(request_context).printable())
        })
    }
}

impl<Ok, Err: Context> WrappedWithInfo<Ok, Err>
    for std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>>
{
    #[track_caller]
    fn into_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>> {
        self
    }
}

impl<Ok> WrappedWithInfo<Ok, InternalApiError> for std::result::Result<Ok, InternalApiError> {
    #[track_caller]
    fn into_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<InternalApiError>>> {
        let value = self?;
        Ok(value)
    }
}

/// Convert to DataError and attach more info to current error
pub trait IntoDataError<Ok, Err: Context>: Sized {
    fn into_data_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>>;

    #[track_caller]
    fn into_data_error<T: Debug + IsLoggingAllowed>(
        self,
        request_context: T,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>> {
        self.into_data_error_without_context().map_err(|e| {
            e.attach_printable(ErrorContext::<T, Ok>::new(request_context).printable())
        })
    }

    #[track_caller]
    fn into_error(self) -> std::result::Result<Ok, WrappedReport<error_stack::Report<Err>>> {
        self.into_data_error_without_context()
    }
}

impl<Ok> IntoDataError<Ok, DataError> for error_stack::Result<Ok, crate::data::file::FileError> {
    #[track_caller]
    fn into_data_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<DataError>>> {
        let value = self?;
        Ok(value)
    }
}

impl<Ok> IntoDataError<Ok, DataError> for error_stack::Result<Ok, crate::data::cache::CacheError> {
    #[track_caller]
    fn into_data_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<DataError>>> {
        let value = self?;
        Ok(value)
    }
}

impl<Ok> IntoDataError<Ok, DataError>
    for error_stack::Result<Ok, simple_backend_database::DataError>
{
    #[track_caller]
    fn into_data_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<DataError>>> {
        let value = self?;
        Ok(value)
    }
}

impl<Ok> IntoDataError<Ok, DataError>
    for error_stack::Result<Ok, simple_backend_database::diesel_db::DieselDatabaseError>
{
    #[track_caller]
    fn into_data_error_without_context(
        self,
    ) -> std::result::Result<Ok, WrappedReport<error_stack::Report<DataError>>> {
        let value = self?;
        Ok(value)
    }
}

/// Absolsute path to database root directory.
#[derive(Clone, Debug)]
pub struct DatabaseRoot {
    root: PathBuf,
    file_dir: FileDir,
}

impl DatabaseRoot {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Self, DataError> {
        let root = path.as_ref().to_path_buf();
        if !root.exists() {
            fs::create_dir(&root)?;
        }

        let file_dir = root.join(DB_FILE_DIR_NAME);
        if !file_dir.exists() {
            fs::create_dir(&file_dir)?;
        }
        let file_dir = FileDir::new(file_dir);

        Ok(Self { root, file_dir })
    }

    pub fn file_dir(&self) -> &FileDir {
        &self.file_dir
    }
}

/// Handle SQLite databases and write command runner.
pub struct DatabaseManager {
    current_read_close: DbReadCloseHandle,
    current_write_close: DbWriteCloseHandle,
    history_read_close: DbReadCloseHandle,
    history_write_close: DbWriteCloseHandle,
}

impl DatabaseManager {
    /// Runs also some blocking file system code.
    pub async fn new<T: AsRef<Path>>(
        database_dir: T,
        config: Arc<Config>,
        media_backup: MediaBackupHandle,
        push_notification_sender: PushNotificationSender,
    ) -> Result<(Self, RouterDatabaseReadHandle, RouterDatabaseWriteHandle), DataError> {
        info!("Creating DatabaseManager");

        let root = DatabaseRoot::new(database_dir)?;

        // Write handles

        let (current_write, current_write_close) =
            DatabaseHandleCreator::create_write_handle_from_config(
                config.simple_backend(),
                "current",
                database::DIESEL_MIGRATIONS,
            )
            .await?;

        let diesel_sqlite = current_write.diesel().sqlite_version().await?;
        info!("Diesel SQLite version: {}", diesel_sqlite);

        let (history_write, history_write_close) =
            DatabaseHandleCreator::create_write_handle_from_config(
                config.simple_backend(),
                "history",
                database::DIESEL_MIGRATIONS,
            )
            .await?;

        // Read handles

        let (current_read, current_read_close) =
            DatabaseHandleCreator::create_read_handle_from_config(
                config.simple_backend(),
                "current",
            )
            .await?;

        let (history_read, history_read_close) =
            DatabaseHandleCreator::create_read_handle_from_config(
                config.simple_backend(),
                "history",
            )
            .await?;

        let index = LocationIndexManager::new(config.clone());
        let current_read_handle = CurrentReadHandle(current_read);
        let current_write_handle = CurrentWriteHandle(current_write);
        let history_read_handle = HistoryReadHandle(history_read);
        let history_write_handle = HistoryWriteHandle(history_write);

        let cache = DatabaseCache::new(&current_read_handle, &index, &config).await?;

        let router_write_handle = RouterDatabaseWriteHandle {
            config: config.clone(),
            current_read_handle: current_read_handle.clone(),
            current_write_handle: current_write_handle.clone(),
            history_read_handle: history_read_handle.clone(),
            history_write_handle: history_write_handle.clone(),
            root: root.into(),
            cache: cache.into(),
            location: index.into(),
            media_backup,
            push_notification_sender,
        };

        let root = router_write_handle.root.clone();
        let cache = router_write_handle.cache.clone();
        let router_read_handle = RouterDatabaseReadHandle {
            current_read_handle: current_read_handle.clone(),
            history_read_handle: history_read_handle.clone(),
            root,
            cache,
        };

        let database_manager = DatabaseManager {
            current_write_close,
            current_read_close,
            history_write_close,
            history_read_close,
        };

        info!("DatabaseManager created");

        Ok((database_manager, router_read_handle, router_write_handle))
    }

    pub async fn close(self) {
        self.current_read_close.close().await;
        self.current_write_close.close().await;
        self.history_read_close.close().await;
        self.history_write_close.close().await;
    }
}

#[derive(Clone, Debug)]
pub struct RouterDatabaseWriteHandle {
    config: Arc<Config>,
    root: Arc<DatabaseRoot>,
    current_read_handle: CurrentReadHandle,
    current_write_handle: CurrentWriteHandle,
    history_read_handle: HistoryReadHandle,
    history_write_handle: HistoryWriteHandle,
    cache: Arc<DatabaseCache>,
    location: Arc<LocationIndexManager>,
    media_backup: MediaBackupHandle,
    push_notification_sender: PushNotificationSender,
}

impl RouterDatabaseWriteHandle {
    pub fn user_write_commands(&self) -> WriteCommands {
        WriteCommands::new(
            &self.config,
            &self.current_write_handle,
            &self.history_write_handle,
            &self.cache,
            &self.root.file_dir,
            &self.location,
            &self.media_backup,
            &self.push_notification_sender,
        )
    }

    pub fn user_write_commands_account<'b>(&'b self) -> WriteCommandsConcurrent<'b> {
        WriteCommandsConcurrent::new(
            &self.current_write_handle,
            &self.history_write_handle,
            &self.cache,
            &self.root.file_dir,
            LocationIndexIteratorHandle::new(&self.location),
        )
    }

    pub fn into_sync_handle(self) -> SyncWriteHandle {
        SyncWriteHandle {
            config: self.config,
            root: self.root,
            current_read_handle: self.current_write_handle.to_read_handle(),
            current_write_handle: self.current_write_handle,
            history_write_handle: self.history_write_handle,
            cache: self.cache,
            location: self.location,
            media_backup: self.media_backup,
            push_notification_sender: self.push_notification_sender,
        }
    }
}

/// Handle for writing synchronous write commands.
#[derive(Clone, Debug)]
pub struct SyncWriteHandle {
    config: Arc<Config>,
    root: Arc<DatabaseRoot>,
    current_write_handle: CurrentWriteHandle,
    current_read_handle: CurrentReadHandle,
    history_write_handle: HistoryWriteHandle,
    cache: Arc<DatabaseCache>,
    location: Arc<LocationIndexManager>,
    media_backup: MediaBackupHandle,
    push_notification_sender: PushNotificationSender,
}

impl SyncWriteHandle {
    fn cmds(&self) -> WriteCommands {
        WriteCommands::new(
            &self.config,
            &self.current_write_handle,
            &self.history_write_handle,
            &self.cache,
            &self.root.file_dir,
            &self.location,
            &self.media_backup,
            &self.push_notification_sender,
        )
    }

    pub fn read(&self) -> ReadCommands<'_> {
        ReadCommands::new(
            &self.current_read_handle,
            &self.cache,
            &self.root.file_dir
        )
    }

    pub fn common(&self) -> WriteCommandsCommon {
        self.cmds().common()
    }

    pub fn account(&self) -> WriteCommandsAccount {
        self.cmds().account()
    }

    pub fn account_admin(&self) -> WriteCommandsAccountAdmin {
        self.cmds().account_admin()
    }

    pub fn media(&self) -> WriteCommandsMedia {
        self.cmds().media()
    }

    pub fn media_admin(&self) -> WriteCommandsMediaAdmin {
        self.cmds().media_admin()
    }

    pub fn profile(&self) -> WriteCommandsProfile {
        self.cmds().profile()
    }

    pub fn profile_admin(&self) -> WriteCommandsProfileAdmin {
        self.cmds().profile_admin()
    }

    pub fn chat(&self) -> WriteCommandsChat {
        self.cmds().chat()
    }

    pub fn chat_admin(&self) -> WriteCommandsChatAdmin {
        self.cmds().chat_admin()
    }

    pub fn events(&self) -> EventManagerWithCacheReference {
        EventManagerWithCacheReference::new(
            &self.cache,
            &self.push_notification_sender,
        )
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn register(
        &self,
        id: AccountId,
        sign_in_with_info: SignInWithInfo,
        email: Option<EmailAddress>,
    ) -> Result<AccountIdInternal, DataError> {
        self.cmds().register(id, sign_in_with_info, email).await
    }
}

pub struct RouterDatabaseReadHandle {
    root: Arc<DatabaseRoot>,
    current_read_handle: CurrentReadHandle,
    history_read_handle: HistoryReadHandle,
    cache: Arc<DatabaseCache>,
}

impl RouterDatabaseReadHandle {
    pub fn read(&self) -> ReadCommands<'_> {
        ReadCommands::new(&self.current_read_handle, &self.cache, &self.root.file_dir)
    }

    pub fn read_files(&self) -> FileReadCommands<'_> {
        FileReadCommands::new(&self.root.file_dir)
    }

    pub fn access_token_manager(&self) -> AccessTokenManager<'_> {
        AccessTokenManager::new(&self.cache)
    }

    pub fn account_id_manager(&self) -> AccountIdManager<'_> {
        AccountIdManager::new(&self.cache)
    }

    pub fn cache(&self) -> &DatabaseCache {
        &self.cache
    }
}
