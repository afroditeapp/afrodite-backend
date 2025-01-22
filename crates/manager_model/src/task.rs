use serde::{Deserialize, Serialize};
use simple_backend_model::UnixTime;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct ScheduledTaskStatus {
    pub system_reboot: Option<MaintenanceTask>,
    pub backend_restart: Option<MaintenanceTask>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct MaintenanceTask {
    pub time: UnixTime,
    pub notify_backend: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, IntoParams)]
pub struct NotifyBackend {
    pub notify_backend: bool,
}
