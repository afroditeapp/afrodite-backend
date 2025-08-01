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
pub enum ProfileModerationContentType {
    #[serde(rename = "ProfileName")]
    ProfileName,
    #[serde(rename = "ProfileText")]
    ProfileText,

}

impl std::fmt::Display for ProfileModerationContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ProfileName => write!(f, "ProfileName"),
            Self::ProfileText => write!(f, "ProfileText"),
        }
    }
}

impl Default for ProfileModerationContentType {
    fn default() -> ProfileModerationContentType {
        Self::ProfileName
    }
}

