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
pub struct PublicKeyIdAndVersion {
    #[serde(rename = "id")]
    pub id: Box<models::PublicKeyId>,
    #[serde(rename = "version")]
    pub version: Box<models::PublicKeyVersion>,
}

impl PublicKeyIdAndVersion {
    pub fn new(id: models::PublicKeyId, version: models::PublicKeyVersion) -> PublicKeyIdAndVersion {
        PublicKeyIdAndVersion {
            id: Box::new(id),
            version: Box::new(version),
        }
    }
}

