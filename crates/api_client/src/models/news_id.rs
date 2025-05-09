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

/// NewsId : News ID
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewsId {
    #[serde(rename = "nid")]
    pub nid: i64,
}

impl NewsId {
    /// News ID
    pub fn new(nid: i64) -> NewsId {
        NewsId {
            nid,
        }
    }
}

