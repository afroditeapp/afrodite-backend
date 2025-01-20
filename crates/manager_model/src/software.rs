use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SoftwareUpdateStatus {
    pub state: SoftwareUpdateState,
    pub downloaded: Option<SoftwareInfoNew>,
    pub installed: Option<SoftwareInfoNew>,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum SoftwareUpdateState {
    Idle,
    Downloading,
    Installing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SoftwareInfoNew {
    pub name: String,
    pub hash: String,
}
