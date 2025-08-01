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
pub enum CustomReportType {
    #[serde(rename = "Empty")]
    Empty,

}

impl std::fmt::Display for CustomReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
        }
    }
}

impl Default for CustomReportType {
    fn default() -> CustomReportType {
        Self::Empty
    }
}

