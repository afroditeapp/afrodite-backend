use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct SoftwareUpdateStatus {
    pub state: SoftwareUpdateState,
    pub downloaded: Option<SoftwareInfo>,
    pub installed: Option<SoftwareInfo>,
}

impl SoftwareUpdateStatus {
    pub fn new_idle() -> Self {
        Self {
            state: SoftwareUpdateState::Idle,
            downloaded: None,
            installed: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub enum SoftwareUpdateState {
    Idle,
    Downloading,
    Installing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, IntoParams)]
pub struct SoftwareInfo {
    pub name: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum SoftwareUpdateTaskType {
    Download,
    Install(SoftwareInfo),
}
