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
pub struct ManagerInstanceNameList {
    #[serde(rename = "names")]
    pub names: Vec<String>,
}

impl ManagerInstanceNameList {
    pub fn new(names: Vec<String>) -> ManagerInstanceNameList {
        ManagerInstanceNameList {
            names,
        }
    }
}

