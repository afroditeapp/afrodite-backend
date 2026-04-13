use std::sync::Arc;

use database::{DbReaderRaw, current::read::GetDbReadCommandsCommon};
use error_stack::ResultExt;
use model::DynamicServerConfig;
use tokio::sync::RwLock;

use crate::DataError;

#[derive(Debug, Clone, Default)]
pub struct DynamicServerConfigManager {
    value: Arc<RwLock<Option<DynamicServerConfig>>>,
}

impl DynamicServerConfigManager {
    pub fn new(value: Option<DynamicServerConfig>) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
        }
    }

    pub async fn dynamic_server_config(&self) -> Option<DynamicServerConfig> {
        self.value.read().await.clone()
    }

    pub fn set_dynamic_server_config_blocking(&self, value: Option<DynamicServerConfig>) {
        *self.value.blocking_write() = value;
    }
}

pub async fn load_dynamic_server_config_from_db(
    reader: &DbReaderRaw<'_>,
) -> error_stack::Result<DynamicServerConfigManager, DataError> {
    let value = reader
        .db_read(move |mut mode| mode.common().client_config().dynamic_server_config())
        .await
        .change_context(DataError::Diesel)?;

    Ok(DynamicServerConfigManager::new(value))
}
