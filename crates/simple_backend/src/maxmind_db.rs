//! MaxMind DB access
//!

use std::{
    net::IpAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime},
};

use error_stack::{Result, ResultExt};
use futures::StreamExt;
use hyper::StatusCode;
use simple_backend_config::{SimpleBackendConfig, file::MaxMindDbConfig};
use simple_backend_database::data::create_dirs_and_get_simple_backend_dir_path;
use simple_backend_model::UnixTime;
use simple_backend_utils::{ContextExt, file::overwrite_and_remove_if_exists};
use tokio::{io::AsyncWriteExt, sync::RwLock, task::JoinHandle};
use tracing::{error, warn};

use crate::ServerQuitWatcher;

#[derive(thiserror::Error, Debug)]
enum MaxMindDbError {
    #[error("Data directory related error")]
    DataDir,

    #[error("Overwrite error")]
    Overwrite,

    #[error("File metadata error")]
    FileMetadata,

    #[error("File rename error")]
    FileRename,

    #[error("Download error")]
    Download,

    #[error("Read error")]
    Read,
}

#[derive(Debug)]
pub struct MaxMindDbManagerQuitHandle {
    task: JoinHandle<()>,
}

impl MaxMindDbManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("MaxMindDbManager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct IpDb {
    db: maxminddb::Reader<Vec<u8>>,
}

impl IpDb {
    pub fn get_country(&self, ip: IpAddr) -> Option<String> {
        match self.db.lookup::<maxminddb::geoip2::Country>(ip) {
            Ok(v) => v
                .and_then(|v| v.country)
                .and_then(|v| v.iso_code)
                .map(|v| v.to_string()),
            Err(e) => {
                error!("MaxMind DB error: {}", e);
                None
            }
        }
    }
}

pub struct MaxMindDbManagerData {
    db: RwLock<Option<Arc<IpDb>>>,
}

impl MaxMindDbManagerData {
    pub fn new() -> Self {
        Self {
            db: RwLock::new(None),
        }
    }

    pub async fn current_db(&self) -> Option<Arc<IpDb>> {
        let lock = self.db.read().await;
        lock.as_ref().cloned()
    }

    async fn is_db_loaded(&self) -> bool {
        let lock = self.db.read().await;
        lock.as_ref().is_some()
    }

    async fn replace_db(&self, db: maxminddb::Reader<Vec<u8>>) {
        let mut lock = self.db.write().await;
        *lock = Some(Arc::new(IpDb { db }));
    }
}

impl Default for MaxMindDbManagerData {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MaxMindDbManager {
    data: Arc<MaxMindDbManagerData>,
    config: Arc<SimpleBackendConfig>,
    client: Arc<reqwest::Client>,
}

impl MaxMindDbManager {
    pub fn new_manager(
        data: Arc<MaxMindDbManagerData>,
        quit_notification: ServerQuitWatcher,
        config: Arc<SimpleBackendConfig>,
        client: Arc<reqwest::Client>,
    ) -> MaxMindDbManagerQuitHandle {
        let manager = Self {
            data,
            config,
            client,
        };

        let task = tokio::spawn(manager.run(quit_notification));

        MaxMindDbManagerQuitHandle { task }
    }

    async fn run(self, mut quit_notification: ServerQuitWatcher) {
        loop {
            tokio::select! {
                _ = self.logic() => (),
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }

    const SECONDS_IN_WEEK: u64 = 60 * 60 * 24 * 7;

    async fn logic(&self) {
        let mut timer = tokio::time::interval(Duration::from_secs(Self::SECONDS_IN_WEEK));
        loop {
            timer.tick().await;
            self.refresh_db_handle_result().await;
        }
    }

    async fn refresh_db_handle_result(&self) {
        match self.refresh_db().await {
            Ok(()) => (),
            Err(e) => error!("MaxMind DB error: {:?}", e),
        }
    }

    async fn refresh_db(&self) -> Result<(), MaxMindDbError> {
        let Some(config) = self.config.maxmind_db_config() else {
            return Ok(());
        };

        let tmp = self.tmp_file()?;
        overwrite_and_remove_if_exists(&tmp)
            .await
            .change_context(MaxMindDbError::Overwrite)?;

        let db = self.db_file()?;
        let reload_db = if db.exists() {
            if self.file_one_week_old_or_older(&db)? {
                self.download_db_file(config).await?;
                true
            } else {
                false
            }
        } else {
            self.download_db_file(config).await?;
            true
        };

        if !self.data.is_db_loaded().await || reload_db {
            self.load_db_file_to_ram().await?;
        }

        Ok(())
    }

    fn tmp_file(&self) -> Result<PathBuf, MaxMindDbError> {
        create_dirs_and_get_simple_backend_dir_path(&self.config)
            .change_context(MaxMindDbError::DataDir)
            .map(|v| v.join("maxmind.db.tmp"))
    }

    fn db_file(&self) -> Result<PathBuf, MaxMindDbError> {
        create_dirs_and_get_simple_backend_dir_path(&self.config)
            .change_context(MaxMindDbError::DataDir)
            .map(|v| v.join("maxmind.db"))
    }

    fn file_one_week_old_or_older(&self, file: &Path) -> Result<bool, MaxMindDbError> {
        // TODO(future): Avoid blocking I/O
        let metadata = file
            .metadata()
            .change_context(MaxMindDbError::FileMetadata)?;
        let file_created_unix_time = metadata
            .created()
            .change_context(MaxMindDbError::FileMetadata)?
            .duration_since(SystemTime::UNIX_EPOCH)
            .change_context(MaxMindDbError::FileMetadata)?
            .as_secs();
        let current_time = *UnixTime::current_time().as_i64() as u64;
        let difference = file_created_unix_time.abs_diff(current_time);
        Ok(file_created_unix_time <= current_time && difference >= Self::SECONDS_IN_WEEK)
    }

    async fn download_db_file(&self, config: &MaxMindDbConfig) -> Result<(), MaxMindDbError> {
        let request = self
            .client
            .get(config.download_url.clone())
            .build()
            .change_context(MaxMindDbError::Download)?;

        let response = self
            .client
            .execute(request)
            .await
            .change_context(MaxMindDbError::Download)?;

        let status = response.status();
        if status != StatusCode::OK {
            return Err(MaxMindDbError::Download.report())
                .attach_printable(format!("HTTP response status: {}", status));
        }

        let tmp = self.tmp_file()?;
        let mut file = tokio::fs::File::create(&tmp)
            .await
            .change_context(MaxMindDbError::Download)?;

        let mut stream = response.bytes_stream();
        while let Some(bytes) = stream.next().await {
            let bytes = bytes.change_context(MaxMindDbError::Download)?;
            file.write_all(&bytes)
                .await
                .change_context(MaxMindDbError::Download)?;
        }

        let db = self.db_file()?;
        overwrite_and_remove_if_exists(&db)
            .await
            .change_context(MaxMindDbError::Overwrite)?;

        tokio::fs::rename(tmp, db)
            .await
            .change_context(MaxMindDbError::FileRename)?;

        Ok(())
    }

    async fn load_db_file_to_ram(&self) -> Result<(), MaxMindDbError> {
        let db = self.db_file()?;
        let db = tokio::task::spawn_blocking(|| {
            maxminddb::Reader::open_readfile(db).change_context(MaxMindDbError::Read)
        })
        .await
        .change_context(MaxMindDbError::Read)??;

        self.data.replace_db(db).await;

        Ok(())
    }
}
