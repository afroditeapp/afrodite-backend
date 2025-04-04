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
pub struct ClientFeaturesConfig {
    #[serde(rename = "features")]
    pub features: Box<models::FeaturesConfig>,
    #[serde(rename = "map")]
    pub map: Box<models::MapConfig>,
}

impl ClientFeaturesConfig {
    pub fn new(features: models::FeaturesConfig, map: models::MapConfig) -> ClientFeaturesConfig {
        ClientFeaturesConfig {
            features: Box::new(features),
            map: Box::new(map),
        }
    }
}

