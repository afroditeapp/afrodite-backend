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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct HandleModerationRequest {
    #[serde(rename = "accept")]
    pub accept: bool,
}

impl HandleModerationRequest {
    pub fn new(accept: bool) -> HandleModerationRequest {
        HandleModerationRequest {
            accept,
        }
    }
}

