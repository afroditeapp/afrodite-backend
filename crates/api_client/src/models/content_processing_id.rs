/*
 * dating-app-backend
 *
 * Dating app backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// ContentProcessingId : Content ID which is queued to be processed
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContentProcessingId {
    #[serde(rename = "id")]
    pub id: String,
}

impl ContentProcessingId {
    /// Content ID which is queued to be processed
    pub fn new(id: String) -> ContentProcessingId {
        ContentProcessingId {
            id,
        }
    }
}

