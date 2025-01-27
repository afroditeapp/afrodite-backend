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
pub struct SendMessageResult {
    #[serde(rename = "error_receiver_blocked_sender_or_receiver_not_found", skip_serializing_if = "Option::is_none")]
    pub error_receiver_blocked_sender_or_receiver_not_found: Option<bool>,
    #[serde(rename = "error_receiver_public_key_outdated", skip_serializing_if = "Option::is_none")]
    pub error_receiver_public_key_outdated: Option<bool>,
    #[serde(rename = "error_too_many_receiver_acknowledgements_missing", skip_serializing_if = "Option::is_none")]
    pub error_too_many_receiver_acknowledgements_missing: Option<bool>,
    #[serde(rename = "error_too_many_sender_acknowledgements_missing", skip_serializing_if = "Option::is_none")]
    pub error_too_many_sender_acknowledgements_missing: Option<bool>,
    /// None if error happened
    #[serde(rename = "mn", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub mn: Option<Option<Box<models::MessageNumber>>>,
    /// None if error happened
    #[serde(rename = "ut", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub ut: Option<Option<Box<models::UnixTime>>>,
}

impl SendMessageResult {
    pub fn new() -> SendMessageResult {
        SendMessageResult {
            error_receiver_blocked_sender_or_receiver_not_found: None,
            error_receiver_public_key_outdated: None,
            error_too_many_receiver_acknowledgements_missing: None,
            error_too_many_sender_acknowledgements_missing: None,
            mn: None,
            ut: None,
        }
    }
}

