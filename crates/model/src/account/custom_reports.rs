use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{CustomReportTypeNumberValue, ReportTypeNumber};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum CustomReportsOrderMode {
    OrderNumber,
}

#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct CustomReportId(u8);

impl CustomReportId {
    /// 63
    const MAX_VALUE: u8 = (ReportTypeNumber::LAST_CUSTOM_REPORT_TYPE_NUMBER
        - ReportTypeNumber::FIRST_CUSTOM_REPORT_TYPE_NUMBER) as u8;

    pub fn new(value: u8) -> Result<Self, String> {
        if value > Self::MAX_VALUE {
            return Err(format!(
                "Custom report ID value {} is too large, max value: {}",
                value,
                Self::MAX_VALUE
            ));
        }
        Ok(Self(value))
    }

    pub fn to_usize(&self) -> usize {
        self.0.into()
    }

    pub fn to_report_type_number_value(&self) -> Result<CustomReportTypeNumberValue, String> {
        CustomReportTypeNumberValue::new(
            self.0 + ReportTypeNumber::FIRST_CUSTOM_REPORT_TYPE_NUMBER as u8,
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CustomReportsConfigHash {
    h: String,
}

impl CustomReportsConfigHash {
    pub fn new(h: String) -> Self {
        Self { h }
    }

    pub fn hash(&self) -> &str {
        &self.h
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum CustomReportType {
    // Empty content
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CustomReportsConfig {
    report_order: CustomReportsOrderMode,
    reports: Vec<CustomReport>,
}

impl CustomReportsConfig {
    pub fn validate_and_sort_by_id(&mut self) -> Result<(), String> {
        let mut keys = HashSet::new();
        let mut ids = HashSet::new();
        let mut order_numbers = HashSet::new();
        // Validate uniquenes of keys, IDs and order numbers.
        for report in &self.reports {
            if keys.contains(&report.key) {
                return Err(format!("Duplicate key {}", report.key));
            }
            keys.insert(report.key.clone());

            if ids.contains(&report.id) {
                return Err(format!("Duplicate id {}", report.id.to_usize()));
            }
            ids.insert(report.id);

            if order_numbers.contains(&report.order_number) {
                return Err(format!("Duplicate order number {}", report.order_number));
            }
            order_numbers.insert(report.order_number);
        }

        // Check that correct IDs are used.
        for i in 0..self.reports.len() {
            let i: u8 = i
                .try_into()
                .map_err(|e: std::num::TryFromIntError| e.to_string())?;
            let id = CustomReportId::new(i)?;
            if !ids.contains(&id) {
                return Err(format!(
                    "ID {} is missing from custom report ID values, all numbers between 0 and {} should be used",
                    i,
                    self.reports.len() - 1
                ));
            }
        }

        for r in &self.reports {
            r.validate()?;
        }

        self.reports.sort_by_key(|a| a.id);

        Ok(())
    }

    pub fn index_with_id(&self, value: CustomReportId) -> Option<&CustomReport> {
        self.reports.get(value.to_usize())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CustomReport {
    pub key: String,
    pub name: String,
    pub report_type: CustomReportType,
    /// Client should show the report type when making a new report.
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub visible: bool,
    pub id: CustomReportId,
    /// Client should order custom reports with this number when
    /// [CustomReportsOrderMode::OrderNumber] is selected.
    pub order_number: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(default = json!([]))]
    pub translations: Vec<CustomReportLanguage>,
}

impl CustomReport {
    fn validate(&self) -> Result<(), String> {
        let mut keys = HashSet::new();
        keys.insert(self.key.clone());

        for t in self.translations.clone() {
            for l in t.values {
                if l.key != self.key {
                    return Err(format!(
                        "Missing custom report key definition for translation key {}",
                        l.key
                    ));
                }
            }
        }

        Ok(())
    }
}

fn value_bool_true() -> bool {
    true
}

fn value_is_true(v: &bool) -> bool {
    *v
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CustomReportLanguage {
    /// Language code.
    pub lang: String,
    pub values: Vec<CustomReportTranslation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CustomReportTranslation {
    /// Custom report name.
    pub key: String,
    /// Translated text.
    pub name: String,
}
