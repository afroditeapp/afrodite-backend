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

/// ReceivedLikesSyncVersion : Sync version for new received likes count
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReceivedLikesSyncVersion {
    #[serde(rename = "version")]
    pub version: i64,
}

impl ReceivedLikesSyncVersion {
    /// Sync version for new received likes count
    pub fn new(version: i64) -> ReceivedLikesSyncVersion {
        ReceivedLikesSyncVersion {
            version,
        }
    }
}

