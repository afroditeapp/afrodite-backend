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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientVersion {
    #[serde(rename = "major")]
    pub major: i32,
    #[serde(rename = "minor")]
    pub minor: i32,
    #[serde(rename = "patch")]
    pub patch: i32,
}

impl ClientVersion {
    pub fn new(major: i32, minor: i32, patch: i32) -> ClientVersion {
        ClientVersion {
            major,
            minor,
            patch,
        }
    }
}

