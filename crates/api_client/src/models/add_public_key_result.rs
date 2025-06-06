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
pub struct AddPublicKeyResult {
    #[serde(rename = "error_too_many_public_keys")]
    pub error_too_many_public_keys: bool,
    #[serde(rename = "key_id", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub key_id: Option<Option<Box<models::PublicKeyId>>>,
}

impl AddPublicKeyResult {
    pub fn new(error_too_many_public_keys: bool) -> AddPublicKeyResult {
        AddPublicKeyResult {
            error_too_many_public_keys,
            key_id: None,
        }
    }
}

