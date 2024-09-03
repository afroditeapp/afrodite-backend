
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_struct_try_from;
use utoipa::ToSchema;

use crate::{
    schema_sqlite_types::Integer, ProfileStateInternal,
};

/// Profile age value which is in inclusive range `[18, 99]`.
///
/// This serializes to i64, so this must not be added to API doc.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Eq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Integer)]
#[serde(try_from = "i64")]
#[serde(into = "i64")]
pub struct ProfileAge {
    value: u8,
}

impl ProfileAge {
    pub const MIN_AGE: u8 = 18;
    pub const MAX_AGE: u8 = 99;

    pub fn new_clamped(age: u8) -> Self {
        Self {
            value: age.clamp(Self::MIN_AGE, Self::MAX_AGE),
        }
    }
    pub fn value(&self) -> u8 {
        self.value
    }
}

impl Default for ProfileAge {
    fn default() -> Self {
        Self {
            value: Self::MIN_AGE,
        }
    }
}

impl TryFrom<i64> for ProfileAge {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < Self::MIN_AGE as i64 || value > Self::MAX_AGE as i64 {
            Err(format!(
                "Profile age must be in range [{}, {}]",
                Self::MIN_AGE,
                Self::MAX_AGE
            ))
        } else {
            Ok(Self { value: value as u8 })
        }
    }
}

impl From<ProfileAge> for i64 {
    fn from(value: ProfileAge) -> Self {
        value.value as i64
    }
}

diesel_i64_struct_try_from!(ProfileAge);

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProfileSearchAgeRange {
    /// Min value for this field is 18.
    pub min: u8,
    /// Max value for this field is 99.
    pub max: u8,
}

impl From<ProfileStateInternal> for ProfileSearchAgeRange {
    fn from(value: ProfileStateInternal) -> Self {
        Self {
            min: value.search_age_range_min.value(),
            max: value.search_age_range_max.value(),
        }
    }
}

/// Profile search age range which min and max are in
/// inclusive range of `[18, 99]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProfileSearchAgeRangeValidated {
    min: ProfileAge,
    max: ProfileAge,
}

impl ProfileSearchAgeRangeValidated {
    /// New range from two values. Automatically orders the values.
    pub fn new(value1: ProfileAge, value2: ProfileAge) -> Self {
        if value1.value() <= value2.value() {
            Self {
                min: value1,
                max: value2,
            }
        } else {
            Self {
                min: value2,
                max: value1,
            }
        }
    }

    pub fn min(&self) -> ProfileAge {
        self.min
    }

    pub fn max(&self) -> ProfileAge {
        self.max
    }

    pub fn is_match(&self, age: ProfileAge) -> bool {
        age.value() >= self.min.value() && age.value() <= self.max.value()
    }
}

impl TryFrom<ProfileSearchAgeRange> for ProfileSearchAgeRangeValidated {
    type Error = String;

    fn try_from(value: ProfileSearchAgeRange) -> Result<Self, Self::Error> {
        if value.min > value.max {
            Err("Min value must be less than or equal to max value".to_string())
        } else {
            let min = (value.min as i64).try_into()?;
            let max = (value.max as i64).try_into()?;
            Ok(Self { min, max })
        }
    }
}
