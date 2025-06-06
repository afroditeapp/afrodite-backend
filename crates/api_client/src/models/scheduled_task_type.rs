/*
 * afrodite-backend
 *
 * Dating app backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ScheduledTaskType {
    #[serde(rename = "BackendRestart")]
    BackendRestart,
    #[serde(rename = "SystemReboot")]
    SystemReboot,

}

impl std::fmt::Display for ScheduledTaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::BackendRestart => write!(f, "BackendRestart"),
            Self::SystemReboot => write!(f, "SystemReboot"),
        }
    }
}

impl Default for ScheduledTaskType {
    fn default() -> ScheduledTaskType {
        Self::BackendRestart
    }
}

