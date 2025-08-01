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

/// AccountBanReasonDetails : This might be empty.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountBanReasonDetails {
    #[serde(rename = "value")]
    pub value: String,
}

impl AccountBanReasonDetails {
    /// This might be empty.
    pub fn new(value: String) -> AccountBanReasonDetails {
        AccountBanReasonDetails {
            value,
        }
    }
}

