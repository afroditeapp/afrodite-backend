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

/// GetApiUsageStatisticsSettings : Time range is inclusive. [Self::max_time] must be greater or equal to [Self::min_time].
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetApiUsageStatisticsSettings {
    #[serde(rename = "account")]
    pub account: Box<models::AccountId>,
    #[serde(rename = "max_time", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub max_time: Option<Option<Box<models::UnixTime>>>,
    #[serde(rename = "min_time", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub min_time: Option<Option<Box<models::UnixTime>>>,
}

impl GetApiUsageStatisticsSettings {
    /// Time range is inclusive. [Self::max_time] must be greater or equal to [Self::min_time].
    pub fn new(account: models::AccountId) -> GetApiUsageStatisticsSettings {
        GetApiUsageStatisticsSettings {
            account: Box::new(account),
            max_time: None,
            min_time: None,
        }
    }
}

