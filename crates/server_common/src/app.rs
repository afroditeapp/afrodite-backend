use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use config::{file::ConfigFileError, file_dynamic::ConfigFileDynamic, Config};
use error_stack::{Result, ResultExt};
use futures::Future;
use model::{AccountId, AccountIdInternal, BackendConfig, BackendVersion};
use simple_backend::{
    app::{GetSimpleBackendConfig, SimpleBackendAppState},
    web_socket::WebSocketManager,
};


use crate::{
    data::DataError, push_notifications::PushNotificationSender
};

pub trait GetConfig {
    fn config(&self) -> &Config;
}

#[async_trait::async_trait]
pub trait WriteDynamicConfig {
    async fn write_config(&self, config: BackendConfig)
        -> error_stack::Result<(), ConfigFileError>;
}

#[async_trait::async_trait]
pub trait ReadDynamicConfig {
    async fn read_config(&self) -> error_stack::Result<BackendConfig, ConfigFileError>;
}

pub trait BackendVersionProvider {
    fn backend_version(&self) -> BackendVersion;
}

/// All accounts registered in the service.
pub trait GetAccounts {
    fn get_internal_id(
        &self,
        id: AccountId
    ) -> impl std::future::Future<Output = Result<AccountIdInternal, DataError>> + Send;
}

// pub trait FileAccessProvider {
//     fn file_access(&self) -> &FileDir;
// }

// impl FileAccessProvider for S {
//     fn file_access(&self) -> &FileDir {
//         &self.business_logic_state().
//     }
// }
