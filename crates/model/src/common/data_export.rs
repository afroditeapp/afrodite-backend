use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::AccountId;

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
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Clone, Copy, Deserialize, ToSchema, PartialEq)]
pub enum DataExportType {
    /// User initiated data export which
    /// doesn't expose information on other users.
    User,
    /// Admin initiated data export which
    /// does expose information on other users.
    Admin,
}

#[derive(Deserialize, ToSchema)]
pub struct PostStartDataExport {
    /// Data reading source account.
    pub source: AccountId,
    pub data_export_type: DataExportType,
}
