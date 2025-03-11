//! HTTP API types and request handlers for all servers.

use std::sync::Arc;

use manager_config::Config;

use crate::server::{backend_manager::BackendManagerHandle, client::ApiManager, link::{backup::server::BackupLinkManagerHandleServer, json_rpc::server::JsonRcpLinkManagerHandleServer}, scheduled_task::ScheduledTaskManagerHandle, task::TaskManagerHandle, update::UpdateManagerHandle};

pub mod server;
pub mod client;
pub mod utils;

// App state getters

pub trait GetConfig {
    fn config(&self) -> &Config;
    fn config_arc(&self) -> Arc<Config>;
}

pub trait GetApiManager {
    fn api_manager(&self) -> ApiManager;
}

pub trait GetUpdateManager {
    fn update_manager(&self) -> &UpdateManagerHandle;
}

pub trait GetTaskManager {
    fn task_manager(&self) -> &TaskManagerHandle;
}

pub trait GetScheduledTaskManager {
    fn scheduled_task_manager(&self) -> &ScheduledTaskManagerHandle;
}

pub trait GetJsonRcpLinkManager {
    fn json_rpc_link_server(&self) -> &JsonRcpLinkManagerHandleServer;
}

pub trait GetBackupLinkManager {
    fn backup_link_server(&self) -> &BackupLinkManagerHandleServer;
}

pub trait GetBackendManager {
    fn backend_manager(&self) -> &BackendManagerHandle;
}
