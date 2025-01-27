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
pub struct GetMyProfileResult {
    /// Account's most recent disconnect time.  If the last seen time is not None, then it is Unix timestamp or -1 if the profile is currently online.
    #[serde(rename = "lst", skip_serializing_if = "Option::is_none")]
    pub lst: Option<i64>,
    #[serde(rename = "name_moderation_state")]
    pub name_moderation_state: models::ProfileNameModerationState,
    #[serde(rename = "p")]
    pub p: Box<models::Profile>,
    #[serde(rename = "sv")]
    pub sv: Box<models::ProfileSyncVersion>,
    #[serde(rename = "text_moderation_info")]
    pub text_moderation_info: Box<models::ProfileTextModerationInfo>,
    #[serde(rename = "v")]
    pub v: Box<models::ProfileVersion>,
}

impl GetMyProfileResult {
    pub fn new(name_moderation_state: models::ProfileNameModerationState, p: models::Profile, sv: models::ProfileSyncVersion, text_moderation_info: models::ProfileTextModerationInfo, v: models::ProfileVersion) -> GetMyProfileResult {
        GetMyProfileResult {
            lst: None,
            name_moderation_state,
            p: Box::new(p),
            sv: Box::new(sv),
            text_moderation_info: Box::new(text_moderation_info),
            v: Box::new(v),
        }
    }
}

