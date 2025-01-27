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

/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum AttributeOrderMode {
    #[serde(rename = "OrderNumber")]
    OrderNumber,

}

impl std::fmt::Display for AttributeOrderMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::OrderNumber => write!(f, "OrderNumber"),
        }
    }
}

impl Default for AttributeOrderMode {
    fn default() -> AttributeOrderMode {
        Self::OrderNumber
    }
}

