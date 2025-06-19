use std::{fmt::Debug, sync::Arc};

use error_stack::{Result, ResultExt};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_utils::ContextExt;
use tokio::sync::Mutex;
use utils::encrypt::{GeneratedKeys, ParsedKeys};

const PRIVATE_KEY_FILE_NAME: &str = "backend_private_key.asc";
const PUBLIC_KEY_FILE_NAME: &str = "backend_public_key.asc";

#[derive(thiserror::Error, Debug)]
pub enum DataSignerError {
    #[error("Read error")]
    Read,

    #[error("Write error")]
    Write,

    #[error("Message encryption error")]
    MessageEncryptionError,

    #[error("Only private key file exists")]
    OnlyPrivateKeyExists,

    #[error("Only public key file exists")]
    OnlyPublicKeyExists,

    #[error("Keys not loaded")]
    KeysNotLoaded,
}

struct State {
    keys: Option<Arc<ParsedKeys>>,
}

pub struct DataSigner {
    state: Arc<Mutex<State>>,
}

impl Debug for DataSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DataSigner")
    }
}

impl Clone for DataSigner {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl DataSigner {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State { keys: None })),
        }
    }

    pub async fn load_or_generate_keys(
        &self,
        config: &SimpleBackendConfig,
    ) -> Result<(), DataSignerError> {
        let mut lock = self.state.lock().await;

        let private_key_path = config.data_dir().join(PRIVATE_KEY_FILE_NAME);
        let public_key_path = config.data_dir().join(PUBLIC_KEY_FILE_NAME);
        let keys = if private_key_path.exists() && public_key_path.exists() {
            let private = tokio::fs::read_to_string(private_key_path)
                .await
                .change_context(DataSignerError::Read)?;
            let public = tokio::fs::read_to_string(public_key_path)
                .await
                .change_context(DataSignerError::Read)?;

            GeneratedKeys { private, public }
        } else if private_key_path.exists() {
            return Err(DataSignerError::OnlyPrivateKeyExists.report());
        } else if public_key_path.exists() {
            return Err(DataSignerError::OnlyPublicKeyExists.report());
        } else {
            let keys = utils::encrypt::generate_keys("Backend".to_string())
                .change_context(DataSignerError::MessageEncryptionError)?;

            tokio::fs::write(private_key_path, &keys.private)
                .await
                .change_context(DataSignerError::Write)?;

            tokio::fs::write(public_key_path, &keys.public)
                .await
                .change_context(DataSignerError::Write)?;

            keys
        };

        let keys = keys
            .to_parsed_keys()
            .change_context(DataSignerError::MessageEncryptionError)?;

        lock.keys = Some(keys.into());

        Ok(())
    }

    pub async fn keys(&self) -> Result<Arc<ParsedKeys>, DataSignerError> {
        let lock = self.state.lock().await;
        let Some(keys) = &lock.keys else {
            return Err(DataSignerError::KeysNotLoaded.report());
        };
        Ok(keys.clone())
    }

    pub async fn verify_and_extract_backend_signed_data(
        &self,
        data: &[u8],
    ) -> Result<Vec<u8>, DataSignerError> {
        let lock = self.state.lock().await;
        let Some(keys) = &lock.keys else {
            return Err(DataSignerError::KeysNotLoaded.report());
        };
        let data = keys
            .verify_signed_message_and_extract_data(data)
            .change_context(DataSignerError::MessageEncryptionError)?;
        Ok(data)
    }
}
