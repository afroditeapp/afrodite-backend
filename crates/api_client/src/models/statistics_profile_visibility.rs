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
pub enum StatisticsProfileVisibility {
    #[serde(rename = "Public")]
    Public,
    #[serde(rename = "Private")]
    Private,
    #[serde(rename = "All")]
    All,

}

impl std::fmt::Display for StatisticsProfileVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "Public"),
            Self::Private => write!(f, "Private"),
            Self::All => write!(f, "All"),
        }
    }
}

impl Default for StatisticsProfileVisibility {
    fn default() -> StatisticsProfileVisibility {
        Self::Public
    }
}

