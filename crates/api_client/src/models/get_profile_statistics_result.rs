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
pub struct GetProfileStatisticsResult {
    #[serde(rename = "account_count_bots_excluded")]
    pub account_count_bots_excluded: i64,
    #[serde(rename = "age_counts")]
    pub age_counts: Box<models::ProfileAgeCounts>,
    #[serde(rename = "connection_statistics")]
    pub connection_statistics: Box<models::ConnectionStatistics>,
    #[serde(rename = "generation_time")]
    pub generation_time: Box<models::UnixTime>,
}

impl GetProfileStatisticsResult {
    pub fn new(account_count_bots_excluded: i64, age_counts: models::ProfileAgeCounts, connection_statistics: models::ConnectionStatistics, generation_time: models::UnixTime) -> GetProfileStatisticsResult {
        GetProfileStatisticsResult {
            account_count_bots_excluded,
            age_counts: Box::new(age_counts),
            connection_statistics: Box::new(connection_statistics),
            generation_time: Box::new(generation_time),
        }
    }
}

