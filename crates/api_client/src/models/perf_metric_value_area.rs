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
pub struct PerfMetricValueArea {
    /// Time value for the first data point. Every next time value is increased with [Self::time_granularity].
    #[serde(rename = "first_time_value")]
    pub first_time_value: Box<models::UnixTime>,
    /// Time granularity for values in between start time and time points.
    #[serde(rename = "time_granularity")]
    pub time_granularity: models::TimeGranularity,
    #[serde(rename = "values")]
    pub values: Vec<i32>,
}

impl PerfMetricValueArea {
    pub fn new(first_time_value: models::UnixTime, time_granularity: models::TimeGranularity, values: Vec<i32>) -> PerfMetricValueArea {
        PerfMetricValueArea {
            first_time_value: Box::new(first_time_value),
            time_granularity,
            values,
        }
    }
}

