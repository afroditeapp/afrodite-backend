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

/// ClientId : ID which client receives from server once. Next value is incremented compared to previous value, so in practice the ID can be used as unique ID even if it can wrap.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientId {
    #[serde(rename = "id")]
    pub id: i64,
}

impl ClientId {
    /// ID which client receives from server once. Next value is incremented compared to previous value, so in practice the ID can be used as unique ID even if it can wrap.
    pub fn new(id: i64) -> ClientId {
        ClientId {
            id,
        }
    }
}

