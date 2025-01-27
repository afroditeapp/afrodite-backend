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
pub struct ResetNewsIteratorResult {
    #[serde(rename = "c")]
    pub c: Box<models::UnreadNewsCount>,
    #[serde(rename = "s")]
    pub s: Box<models::NewsIteratorSessionId>,
    #[serde(rename = "v")]
    pub v: Box<models::NewsSyncVersion>,
}

impl ResetNewsIteratorResult {
    pub fn new(c: models::UnreadNewsCount, s: models::NewsIteratorSessionId, v: models::NewsSyncVersion) -> ResetNewsIteratorResult {
        ResetNewsIteratorResult {
            c: Box::new(c),
            s: Box::new(s),
            v: Box::new(v),
        }
    }
}

