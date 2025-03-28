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
pub struct AllMatchesPage {
    #[serde(rename = "profiles")]
    pub profiles: Vec<models::AccountId>,
    /// This version can be sent to the server when WebSocket protocol data sync is happening.
    #[serde(rename = "version")]
    pub version: Box<models::MatchesSyncVersion>,
}

impl AllMatchesPage {
    pub fn new(profiles: Vec<models::AccountId>, version: models::MatchesSyncVersion) -> AllMatchesPage {
        AllMatchesPage {
            profiles,
            version: Box::new(version),
        }
    }
}

