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

/// LastSeenTimeFilter : Filter value for last seen time.  Possible values: - Value -1 is show only profiles which are online. - Zero and positive values are max seconds since the profile has been online.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct LastSeenTimeFilter {
    #[serde(rename = "value")]
    pub value: i64,
}

impl LastSeenTimeFilter {
    /// Filter value for last seen time.  Possible values: - Value -1 is show only profiles which are online. - Zero and positive values are max seconds since the profile has been online.
    pub fn new(value: i64) -> LastSeenTimeFilter {
        LastSeenTimeFilter {
            value,
        }
    }
}

