/*
 * dating-app-backend
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
pub struct ProfileAttributeFilterList {
    #[serde(rename = "filters")]
    pub filters: Vec<models::ProfileAttributeFilterValue>,
    #[serde(rename = "last_seen_time_filter", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub last_seen_time_filter: Option<Option<Box<models::LastSeenTimeFilter>>>,
    #[serde(rename = "unlimited_likes_filter", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub unlimited_likes_filter: Option<Option<bool>>,
}

impl ProfileAttributeFilterList {
    pub fn new(filters: Vec<models::ProfileAttributeFilterValue>) -> ProfileAttributeFilterList {
        ProfileAttributeFilterList {
            filters,
            last_seen_time_filter: None,
            unlimited_likes_filter: None,
        }
    }
}

