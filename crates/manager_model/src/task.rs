use serde::{Deserialize, Serialize};
use simple_backend_model::UnixTime;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct ScheduledTaskStatus {
    pub system_reboot: Option<MaintenanceTask>,
    pub backend_restart: Option<MaintenanceTask>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct MaintenanceTask {
    pub time: UnixTime,
    pub notify_backend: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, IntoParams)]
pub struct NotifyBackend {
    pub notify_backend: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum ManualTaskType {
    BackendDataReset,
    BackendRestart,
    SystemReboot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum ScheduledTaskType {
    BackendRestart,
    SystemReboot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ToSchema, IntoParams)]
pub struct ScheduledTaskTypeValue {
    pub scheduled_task_type: ScheduledTaskType,
}
