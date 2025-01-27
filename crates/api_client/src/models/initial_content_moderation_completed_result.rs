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
pub struct InitialContentModerationCompletedResult {
    #[serde(rename = "accepted")]
    pub accepted: bool,
}

impl InitialContentModerationCompletedResult {
    pub fn new(accepted: bool) -> InitialContentModerationCompletedResult {
        InitialContentModerationCompletedResult {
            accepted,
        }
    }
}

