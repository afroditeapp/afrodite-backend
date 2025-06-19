use diesel::{
    deserialize::FromSqlRow, expression::AsExpression, prelude::Queryable, sql_types::BigInt,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{UnixTime, diesel_i64_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::StatisticsGender;

/// Profile statistics save time ID
#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Default, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct SaveTimeId {
    pub id: i64,
}

impl SaveTimeId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }
}

diesel_i64_wrapper!(SaveTimeId);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, IntoParams)]
pub struct GetProfileStatisticsHistoryParams {
    pub value_type: ProfileStatisticsHistoryValueType,
    /// Required only for AgeChange history
    pub age: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileStatisticsHistoryResult {
    pub values: Vec<ProfileStatisticsHistoryValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum ProfileStatisticsHistoryValueType {
    Accounts,
    Public,
    PublicMan,
    PublicWoman,
    PublicNonBinary,
    AgeChange,
    AgeChangeMan,
    AgeChangeWoman,
    AgeChangeNonBinary,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable)]
pub struct ProfileStatisticsHistoryValue {
    pub ut: UnixTime,
    pub c: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProfileStatisticsHistoryValueTypeInternal {
    Accounts,
    Public {
        gender: Option<StatisticsGender>,
    },
    AgeChange {
        gender: Option<StatisticsGender>,
        age: i64,
    },
}

impl TryFrom<GetProfileStatisticsHistoryParams> for ProfileStatisticsHistoryValueTypeInternal {
    type Error = &'static str;
    fn try_from(value: GetProfileStatisticsHistoryParams) -> Result<Self, Self::Error> {
        use ProfileStatisticsHistoryValueType as V;
        let internal = match (value.value_type, value.age) {
            (V::Accounts, _) => Self::Accounts,
            (V::Public, _) => Self::Public { gender: None },
            (V::PublicMan, _) => Self::Public {
                gender: Some(StatisticsGender::Man),
            },
            (V::PublicWoman, _) => Self::Public {
                gender: Some(StatisticsGender::Woman),
            },
            (V::PublicNonBinary, _) => Self::Public {
                gender: Some(StatisticsGender::NonBinary),
            },
            (V::AgeChange, Some(age)) => Self::AgeChange { gender: None, age },
            (V::AgeChangeMan, Some(age)) => Self::AgeChange {
                gender: Some(StatisticsGender::Man),
                age,
            },
            (V::AgeChangeWoman, Some(age)) => Self::AgeChange {
                gender: Some(StatisticsGender::Woman),
                age,
            },
            (V::AgeChangeNonBinary, Some(age)) => Self::AgeChange {
                gender: Some(StatisticsGender::NonBinary),
                age,
            },
            (_, None) => return Err("AgeChange history values require age value"),
        };
        Ok(internal)
    }
}
