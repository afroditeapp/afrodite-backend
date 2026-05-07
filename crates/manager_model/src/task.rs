use serde::{Deserialize, Serialize};
use simple_backend_model::UnixTime;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
pub struct ManagerApiScheduledTaskStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_reboot: Option<ManagerApiMaintenanceTask>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_restart: Option<ManagerApiMaintenanceTask>,
}

impl From<ManagerApiScheduledTaskStatus> for ScheduledTaskStatus {
    fn from(v: ManagerApiScheduledTaskStatus) -> Self {
        Self {
            system_reboot: v.system_reboot.map(Into::into),
            server_restart: v.backend_restart.map(Into::into),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct ScheduledTaskStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    system_reboot: Option<MaintenanceTask>,
    #[serde(skip_serializing_if = "Option::is_none")]
    server_restart: Option<MaintenanceTask>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub struct ManagerApiMaintenanceTask {
    pub time: UnixTime,
    pub notify_backend: bool,
}

impl From<ManagerApiMaintenanceTask> for MaintenanceTask {
    fn from(v: ManagerApiMaintenanceTask) -> Self {
        Self {
            time: v.time,
            notify_server: v.notify_backend,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct MaintenanceTask {
    time: UnixTime,
    notify_server: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagerApiNotifyBackend {
    pub notify_backend: bool,
}

impl From<NotifyServer> for ManagerApiNotifyBackend {
    fn from(v: NotifyServer) -> Self {
        Self {
            notify_backend: v.notify_server,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, IntoParams)]
pub struct NotifyServer {
    notify_server: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ManagerApiManualTaskType {
    BackendDataReset,
    BackendRestart,
    SystemReboot,
    SystemShutdown,
}

impl From<ManualTaskType> for ManagerApiManualTaskType {
    fn from(v: ManualTaskType) -> Self {
        match v {
            ManualTaskType::ServerDataReset => Self::BackendDataReset,
            ManualTaskType::ServerRestart => Self::BackendRestart,
            ManualTaskType::SystemReboot => Self::SystemReboot,
            ManualTaskType::SystemShutdown => Self::SystemShutdown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum ManualTaskType {
    ServerDataReset,
    ServerRestart,
    SystemReboot,
    SystemShutdown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ManagerApiScheduledTaskType {
    BackendRestart,
    SystemReboot,
}

impl From<ScheduledTaskType> for ManagerApiScheduledTaskType {
    fn from(v: ScheduledTaskType) -> Self {
        match v {
            ScheduledTaskType::ServerRestart => Self::BackendRestart,
            ScheduledTaskType::SystemReboot => Self::SystemReboot,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum ScheduledTaskType {
    ServerRestart,
    SystemReboot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ToSchema, IntoParams)]
pub struct ScheduledTaskTypeValue {
    pub scheduled_task_type: ScheduledTaskType,
}
