/*
 * pihka-backend
 *
 * Pihka backend API
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PerfValueArea {
    /// Time for first data point in values.
    #[serde(rename = "start_time")]
    pub start_time: Box<models::UnixTime>,
    /// Time granularity for values in between start time and time points.
    #[serde(rename = "time_granularity")]
    pub time_granularity: models::TimeGranularity,
    #[serde(rename = "values")]
    pub values: Vec<i32>,
}

impl PerfValueArea {
    pub fn new(start_time: models::UnixTime, time_granularity: models::TimeGranularity, values: Vec<i32>) -> PerfValueArea {
        PerfValueArea {
            start_time: Box::new(start_time),
            time_granularity,
            values,
        }
    }
}

