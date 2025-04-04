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
pub struct MapConfig {
    /// Limit viewable map area
    #[serde(rename = "bounds")]
    pub bounds: Box<models::MapBounds>,
    #[serde(rename = "initial_location")]
    pub initial_location: Box<models::MapCoordinate>,
    #[serde(rename = "zoom")]
    pub zoom: Box<models::MapZoom>,
}

impl MapConfig {
    pub fn new(bounds: models::MapBounds, initial_location: models::MapCoordinate, zoom: models::MapZoom) -> MapConfig {
        MapConfig {
            bounds: Box::new(bounds),
            initial_location: Box::new(initial_location),
            zoom: Box::new(zoom),
        }
    }
}

