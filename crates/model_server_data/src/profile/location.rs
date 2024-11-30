use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Default)]
#[serde(try_from = "f64")]
#[serde(into = "f64")]
pub struct FiniteDouble {
    value: f64,
}

impl TryFrom<f64> for FiniteDouble {
    type Error = String;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self { value })
        } else {
            Err("Value must be finite".to_string())
        }
    }
}

impl From<FiniteDouble> for f64 {
    fn from(value: FiniteDouble) -> Self {
        value.value
    }
}

/// Location in latitude and longitude.
/// The values are not NaN, infinity or negative infinity.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Default,
    Queryable,
    Selectable,
    AsChangeset,
)]
#[diesel(table_name = crate::schema::profile_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct Location {
    #[schema(value_type = f64)]
    #[diesel(deserialize_as = f64, serialize_as = f64)]
    latitude: FiniteDouble,
    #[schema(value_type = f64)]
    #[diesel(deserialize_as = f64, serialize_as = f64)]
    longitude: FiniteDouble,
}

impl Location {
    pub fn latitude(&self) -> f64 {
        self.latitude.into()
    }

    pub fn longitude(&self) -> f64 {
        self.longitude.into()
    }
}
