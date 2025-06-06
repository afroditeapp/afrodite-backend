use model_server_data::ProfileSearchAgeRangeValidated;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::ProfileStateInternal;

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

impl TryFrom<ProfileSearchAgeRange> for ProfileSearchAgeRangeValidated {
    type Error = String;

    fn try_from(value: ProfileSearchAgeRange) -> Result<Self, Self::Error> {
        if value.min > value.max {
            Err("Min value must be less than or equal to max value".to_string())
        } else {
            let min = (value.min as i64).try_into()?;
            let max = (value.max as i64).try_into()?;
            Ok(Self::new(min, max))
        }
    }
}
