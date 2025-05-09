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
pub struct SetMaxPublicKeyCount {
    #[serde(rename = "account")]
    pub account: Box<models::AccountId>,
    #[serde(rename = "count")]
    pub count: i64,
}

impl SetMaxPublicKeyCount {
    pub fn new(account: models::AccountId, count: i64) -> SetMaxPublicKeyCount {
        SetMaxPublicKeyCount {
            account: Box::new(account),
            count,
        }
    }
}

