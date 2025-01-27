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
pub struct CommandOutput {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "output")]
    pub output: String,
}

impl CommandOutput {
    pub fn new(name: String, output: String) -> CommandOutput {
        CommandOutput {
            name,
            output,
        }
    }
}

