use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SoftwareUpdateStatus {
    pub state: SoftwareUpdateState,
    pub downloaded: Option<SoftwareInfoNew>,
    pub installed: Option<SoftwareInfoNew>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum SoftwareUpdateState {
    Idle,
    Downloading,
    Installing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SoftwareInfoNew {
    pub file_name: String,
    pub sha256: String,
}
