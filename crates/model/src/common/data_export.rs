use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub enum DataExportStateType {
    Empty,
    InProgress,
    Done,
    Error,
}

impl Default for DataExportStateType {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams, PartialEq)]
pub struct DataExportName {
    pub name: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct DataExportState {
    /// Available when current state is [DataExportStateType::Done].
    pub name: Option<DataExportName>,
    pub state: DataExportStateType,
}

impl DataExportState {
    pub fn empty() -> Self {
        Self {
            name: None,
            state: DataExportStateType::Empty,
        }
    }

    pub fn in_progress() -> Self {
        Self {
            name: None,
            state: DataExportStateType::InProgress,
        }
    }

    pub fn done(name: DataExportName) -> Self {
        Self {
            name: Some(name),
            state: DataExportStateType::Done,
        }
    }

    pub fn error() -> Self {
        Self {
            name: None,
            state: DataExportStateType::Error,
        }
    }
}
