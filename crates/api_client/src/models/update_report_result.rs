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
pub struct UpdateReportResult {
    #[serde(rename = "error_outdated_report_content", skip_serializing_if = "Option::is_none")]
    pub error_outdated_report_content: Option<bool>,
    #[serde(rename = "error_too_many_reports", skip_serializing_if = "Option::is_none")]
    pub error_too_many_reports: Option<bool>,
}

impl UpdateReportResult {
    pub fn new() -> UpdateReportResult {
        UpdateReportResult {
            error_outdated_report_content: None,
            error_too_many_reports: None,
        }
    }
}

