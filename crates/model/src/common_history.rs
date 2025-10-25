use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use simple_backend_model::diesel_i64_wrapper;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Copy, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = BigInt)]
pub struct StatisticsSaveTimeId {
    pub id: i64,
}

impl TryFrom<i64> for StatisticsSaveTimeId {
    type Error = String;

    fn try_from(id: i64) -> Result<Self, Self::Error> {
        Ok(Self { id })
    }
}

impl AsRef<i64> for StatisticsSaveTimeId {
    fn as_ref(&self) -> &i64 {
        &self.id
    }
}

diesel_i64_wrapper!(StatisticsSaveTimeId);
