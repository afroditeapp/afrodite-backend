use std::sync::Arc;

use database::{DbReaderRaw, current::read::GetDbReadCommandsCommon};
use error_stack::ResultExt;
use model::{DynamicClientFeaturesConfig, DynamicClientFeaturesConfigHash};
use tokio::sync::RwLock;

use crate::DataError;

#[derive(Debug, Clone)]
pub struct DynamicClientFeatures {
    pub hash: DynamicClientFeaturesConfigHash,
    pub config: DynamicClientFeaturesConfig,
}

#[derive(Debug, Clone)]
pub struct DynamicClientFeaturesManager {
    value: Arc<RwLock<Option<DynamicClientFeatures>>>,
}

impl Default for DynamicClientFeaturesManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DynamicClientFeaturesManager {
    pub fn new(value: Option<DynamicClientFeatures>) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
        }
    }

    pub async fn dynamic_client_features_hash(&self) -> Option<DynamicClientFeaturesConfigHash> {
        self.value
            .read()
            .await
            .as_ref()
            .map(|value| value.hash.clone())
    }

    pub async fn dynamic_client_features(&self) -> Option<DynamicClientFeatures> {
        self.value.read().await.clone()
    }

    pub async fn set_dynamic_client_features(&self, value: Option<DynamicClientFeatures>) {
        *self.value.write().await = value;
    }
}

pub async fn load_dynamic_client_features_from_db(
    reader: &DbReaderRaw<'_>,
) -> error_stack::Result<DynamicClientFeaturesManager, DataError> {
    let value = reader
        .db_read(move |mut mode| mode.common().client_config().dynamic_client_features())
        .await
        .change_context(DataError::Diesel)?;

    let value = value.map(|(hash, config)| DynamicClientFeatures { hash, config });

    Ok(DynamicClientFeaturesManager::new(value))
}
