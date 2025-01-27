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
pub struct LoginResult {
    /// If `None`, the client is unsupported.
    #[serde(rename = "account", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub account: Option<Option<Box<models::AuthPair>>>,
    /// Account ID of current account. If `None`, the client is unsupported.
    #[serde(rename = "aid", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub aid: Option<Option<Box<models::AccountId>>>,
    #[serde(rename = "email", skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(rename = "error_unsupported_client", skip_serializing_if = "Option::is_none")]
    pub error_unsupported_client: Option<bool>,
    /// Info about latest public keys. Client can use this value to ask if user wants to copy existing private and public key from other device. If empty, public key is not set or the client is unsupported.
    #[serde(rename = "latest_public_keys", skip_serializing_if = "Option::is_none")]
    pub latest_public_keys: Option<Vec<models::PublicKeyIdAndVersion>>,
    /// If `None`, media microservice is disabled or the client version is unsupported.
    #[serde(rename = "media", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub media: Option<Option<Box<models::AuthPair>>>,
    /// If `None`, profile microservice is disabled or the version client is unsupported.
    #[serde(rename = "profile", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub profile: Option<Option<Box<models::AuthPair>>>,
}

impl LoginResult {
    pub fn new() -> LoginResult {
        LoginResult {
            account: None,
            aid: None,
            email: None,
            error_unsupported_client: None,
            latest_public_keys: None,
            media: None,
            profile: None,
        }
    }
}

