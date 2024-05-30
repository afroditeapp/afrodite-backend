

use std::{fmt::Debug, fs, path::Path, sync::Arc};

use config::Config;
use database::{CurrentReadHandle, CurrentWriteHandle, DatabaseHandleCreator, DbReadCloseHandle, DbWriteCloseHandle, HistoryReadHandle, HistoryWriteHandle};
use crate::{event::EventManagerWithCacheReference, write::WriteCommandsContainer};
use model::{AccountId, AccountIdInternal, EmailAddress, SignInWithInfo};
pub use server_common::{
    data::{DataError, IntoDataError},
    result,
};
use server_common::{push_notifications::PushNotificationSender, result::Result};
use simple_backend::media_backup::MediaBackupHandle;
use tracing::info;

use crate::{
    cache::{CacheError, DatabaseCache},
    file::{utils::FileDir, FileError},
    index::{LocationIndexIteratorHandle, LocationIndexManager},
    read::ReadCommands,
    utils::{AccessTokenManager, AccountIdManager},
    write::{
        common::WriteCommandsCommon,
        WriteCommands,
    },
    write_concurrent::WriteCommandsConcurrent,
};


pub const DB_FILE_DIR_NAME: &str = "files";


/// Absolsute path to database root directory.
#[derive(Clone, Debug)]
pub struct DatabaseRoot {
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

        Ok(Self { file_dir })
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

        // let cache = DatabaseCache::new(&current_read_handle, &index,
        // &config).await?;
        let cache = DatabaseCache::new();

        let router_write_handle = RouterDatabaseWriteHandle {
            config: config.clone(),
            current_write_handle: current_write_handle.clone(),
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
            _history_read_handle: history_read_handle.clone(),
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
    current_write_handle: CurrentWriteHandle,
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

    pub fn user_write_commands_account(&self) -> WriteCommandsConcurrent {
        WriteCommandsConcurrent::new(
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
    pub fn cmds(&self) -> WriteCommands {
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
        ReadCommands::new(&self.current_read_handle, &self.cache, &self.root.file_dir)
    }

    pub fn common(&self) -> WriteCommandsCommon<WriteCommandsContainer<'_>> {
        self.cmds().into_common()
    }

    pub fn events(&self) -> EventManagerWithCacheReference<'_> {
        EventManagerWithCacheReference::new(&self.cache, &self.push_notification_sender)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    // pub async fn register(
    //     &self,
    //     id: AccountId,
    //     sign_in_with_info: SignInWithInfo,
    //     email: Option<EmailAddress>,
    // ) -> Result<AccountIdInternal, DataError> {
    //     self.cmds().register(id, sign_in_with_info, email).await
    // }
}

pub struct RouterDatabaseReadHandle {
    root: Arc<DatabaseRoot>,
    current_read_handle: CurrentReadHandle,
    // TODO(prod): Remove if not used
    _history_read_handle: HistoryReadHandle,
    cache: Arc<DatabaseCache>,
}

impl RouterDatabaseReadHandle {
    pub fn read(&self) -> ReadCommands<'_> {
        ReadCommands::new(&self.current_read_handle, &self.cache, &self.root.file_dir)
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


// pub fn account(&self) -> WriteCommandsAccount {
//     self.cmds().account()
// }

// pub fn account_admin(&self) -> WriteCommandsAccountAdmin {
//     self.cmds().account_admin()
// }

// pub fn media(&self) -> WriteCommandsMedia {
//     self.cmds().media()
// }

// pub fn media_admin(&self) -> WriteCommandsMediaAdmin {
//     self.cmds().media_admin()
// }

// pub fn profile(&self) -> WriteCommandsProfile {
//     self.cmds().profile()
// }

// pub fn profile_admin(&self) -> WriteCommandsProfileAdmin {
//     self.cmds().profile_admin()
// }

// pub fn chat(&self) -> WriteCommandsChat {
//     self.cmds().chat()
// }

// pub fn chat_admin(&self) -> WriteCommandsChatAdmin {
//     self.cmds().chat_admin()
// }
