use std::sync::Arc;

use manager_config::Config;
use manager_model::{ServerEvent, ServerEventType};
use tokio::sync::watch;

use super::{
    backend_events::BackendEventsHandle,
    backend_manager::BackendManagerHandle,
    client::ApiManager,
    link::{
        backup::server::BackupLinkManagerHandleServer,
        json_rpc::server::JsonRcpLinkManagerHandleServer,
    },
    scheduled_task::ScheduledTaskManagerHandle,
    task::TaskManagerHandle,
    update::UpdateManagerHandle,
};
use crate::api::{
    GetApiManager, GetBackendManager, GetBackupLinkManager, GetConfig, GetJsonRcpLinkManager,
    GetScheduledTaskManager, GetTaskManager, GetUpdateManager,
};

pub type S = AppState;

#[derive(Debug, Clone)]
pub struct AppState {
    config: Arc<Config>,
    update_manager: Arc<UpdateManagerHandle>,
    task_manager: Arc<TaskManagerHandle>,
    scheduled_task_manager: Arc<ScheduledTaskManagerHandle>,
    backend_events: Arc<BackendEventsHandle>,
    json_rpc_link_handle_server: Arc<JsonRcpLinkManagerHandleServer>,
    backup_link_handle_server: Arc<BackupLinkManagerHandleServer>,
    backend_manager: Arc<BackendManagerHandle>,
}

impl AppState {
    async fn current_state_as_server_events(&self) -> Vec<ServerEvent> {
        let event = ServerEvent {
            event: ServerEventType::MaintenanceSchedulingStatus(
                self.scheduled_task_manager
                    .maintenance_time_for_backend_event()
                    .await,
            ),
        };
        vec![event]
    }

    pub async fn refresh_state_to_backend(&self) {
        self.backend_events
            .send(self.current_state_as_server_events().await);
    }

    pub fn backend_events_receiver(&self) -> watch::Receiver<Vec<ServerEvent>> {
        self.backend_events.receiver()
    }
}

impl GetConfig for AppState {
    fn config(&self) -> &Config {
        &self.config
    }

    fn config_arc(&self) -> Arc<Config> {
        self.config.clone()
    }
}

impl GetUpdateManager for AppState {
    fn update_manager(&self) -> &super::update::UpdateManagerHandle {
        &self.update_manager
    }
}

impl GetTaskManager for AppState {
    fn task_manager(&self) -> &TaskManagerHandle {
        &self.task_manager
    }
}

impl GetScheduledTaskManager for AppState {
    fn scheduled_task_manager(&self) -> &ScheduledTaskManagerHandle {
        &self.scheduled_task_manager
    }
}

impl GetApiManager for AppState {
    fn api_manager(&self) -> super::client::ApiManager {
        ApiManager::new(self)
    }
}

impl GetJsonRcpLinkManager for AppState {
    fn json_rpc_link_server(
        &self,
    ) -> &super::link::json_rpc::server::JsonRcpLinkManagerHandleServer {
        &self.json_rpc_link_handle_server
    }
}

impl GetBackupLinkManager for AppState {
    fn backup_link_server(&self) -> &super::link::backup::server::BackupLinkManagerHandleServer {
        &self.backup_link_handle_server
    }
}

impl GetBackendManager for AppState {
    fn backend_manager(&self) -> &BackendManagerHandle {
        &self.backend_manager
    }
}

pub struct App {
    pub state: AppState,
}

impl App {
    pub async fn new(
        config: Arc<Config>,
        update_manager: Arc<UpdateManagerHandle>,
        task_manager: Arc<TaskManagerHandle>,
        scheduled_task_manager: Arc<ScheduledTaskManagerHandle>,
        json_rpc_link_handle_server: Arc<JsonRcpLinkManagerHandleServer>,
        backup_link_handle_server: Arc<BackupLinkManagerHandleServer>,
        backend_manager: Arc<BackendManagerHandle>,
    ) -> Self {
        let state = AppState {
            config: config.clone(),
            update_manager,
            task_manager,
            scheduled_task_manager,
            backend_events: BackendEventsHandle::new(vec![]).into(),
            json_rpc_link_handle_server,
            backup_link_handle_server,
            backend_manager,
        };

        state.refresh_state_to_backend().await;

        Self { state }
    }

    pub fn state(&self) -> AppState {
        self.state.clone()
    }
}
