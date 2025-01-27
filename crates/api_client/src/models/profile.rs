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

/// Profile : Public profile info
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    #[serde(rename = "age")]
    pub age: i64,
    #[serde(rename = "attributes", skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<models::ProfileAttributeValue>>,
    #[serde(rename = "name")]
    pub name: String,
    /// The name has been accepted using allowlist or manual moderation.
    #[serde(rename = "name_accepted", skip_serializing_if = "Option::is_none")]
    pub name_accepted: Option<bool>,
    /// Profile text support is disabled for now.
    #[serde(rename = "ptext", skip_serializing_if = "Option::is_none")]
    pub ptext: Option<String>,
    /// The profile text has been accepted by bot or human moderator.
    #[serde(rename = "ptext_accepted", skip_serializing_if = "Option::is_none")]
    pub ptext_accepted: Option<bool>,
    #[serde(rename = "unlimited_likes", skip_serializing_if = "Option::is_none")]
    pub unlimited_likes: Option<bool>,
}

impl Profile {
    /// Public profile info
    pub fn new(age: i64, name: String) -> Profile {
        Profile {
            age,
            attributes: None,
            name,
            name_accepted: None,
            ptext: None,
            ptext_accepted: None,
            unlimited_likes: None,
        }
    }
}

