use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use simple_backend_model::diesel_i64_wrapper;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Copy, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = BigInt)]
pub struct StatisticsSaveTimeId {
    pub id: i64,
}

impl StatisticsSaveTimeId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }
}

diesel_i64_wrapper!(StatisticsSaveTimeId);
