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
pub struct GetProfileNameState {
    /// If empty, the profile name is not set.
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "state")]
    pub state: models::ProfileNameModerationState,
}

impl GetProfileNameState {
    pub fn new(name: String, state: models::ProfileNameModerationState) -> GetProfileNameState {
        GetProfileNameState {
            name,
            state,
        }
    }
}

