
/// Type must have new() and to_uuid() methods. Also diesel::FromSqlRow and
/// diesel::AsExpression derives are needed.
///
/// ```
/// #[derive(
///     diesel::FromSqlRow,
///     diesel::AsExpression,
/// )]
/// #[diesel(sql_type = Binary)]
/// pub struct UuidWrapper {
///     uuid: uuid::Uuid,
/// }
///
/// impl UuidWrapper {
///     pub fn new(uuid: uuid::Uuid) -> Self {
///         Self { uuid }
///     }
///
///     pub fn to_uuid(&self) -> uuid::Uuid {
///         self.uuid
///     }
/// }
///
/// diesel_uuid_wrapper!(UuidWrapper);
///
/// ```
///
#[macro_export]
macro_rules! diesel_uuid_wrapper {
    ($name:ty) => {
        impl<DB: diesel::backend::Backend>
            diesel::deserialize::FromSql<diesel::sql_types::Binary, DB> for $name
        where
            Vec<u8>: diesel::deserialize::FromSql<diesel::sql_types::Binary, DB>,
        {
            fn from_sql(
                bytes: <DB as diesel::backend::Backend>::RawValue<'_>,
            ) -> diesel::deserialize::Result<Self> {
                let bytes = Vec::<u8>::from_sql(bytes)?;
                let uuid = uuid::Uuid::from_slice(&bytes)?;
                Ok(<$name>::new(uuid))
            }
        }

        impl<DB: diesel::backend::Backend> diesel::serialize::ToSql<diesel::sql_types::Binary, DB>
            for $name
        where
            [u8]: diesel::serialize::ToSql<diesel::sql_types::Binary, DB>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, DB>,
            ) -> diesel::serialize::Result {
                let uuid = self.as_uuid();
                let bytes = uuid.as_bytes();
                bytes.to_sql(out)
            }
        }
    };
}

pub(crate) use diesel_uuid_wrapper;

/// Type must have new() and as_str() methods.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// #[derive(
///     diesel::FromSqlRow,
///     diesel::AsExpression,
/// )]
/// #[diesel(sql_type = Text)]
/// pub struct StringWrapper {
///     text: String,
/// }
///
/// impl StringWrapper {
///     pub fn new(text: String) -> Self {
///         Self { text }
///     }
///
///     pub fn as_str(&self) -> &str {
///        &self.text
///     }
/// }
///
/// diesel_string_wrapper!(StringWrapper);
///
/// ```
#[macro_export]
macro_rules! diesel_string_wrapper {
    ($name:ty) => {
        impl<DB: diesel::backend::Backend> diesel::deserialize::FromSql<diesel::sql_types::Text, DB>
            for $name
        where
            String: diesel::deserialize::FromSql<diesel::sql_types::Text, DB>,
        {
            fn from_sql(
                value: <DB as diesel::backend::Backend>::RawValue<'_>,
            ) -> diesel::deserialize::Result<Self> {
                let string = String::from_sql(value)?;
                Ok(<$name>::new(string))
            }
        }

        impl<DB: diesel::backend::Backend> diesel::serialize::ToSql<diesel::sql_types::Text, DB>
            for $name
        where
            str: diesel::serialize::ToSql<diesel::sql_types::Text, DB>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, DB>,
            ) -> diesel::serialize::Result {
                self.as_str().to_sql(out)
            }
        }
    };
}

pub(crate) use diesel_string_wrapper;

/// Type must have new() and as_i64() methods.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// #[derive(
///     diesel::FromSqlRow,
///     diesel::AsExpression,
/// )]
/// #[diesel(sql_type = Integer)]
/// pub struct NumberWrapper {
///     number: i64,
/// }
///
/// impl NumberWrapper {
///     pub fn new(number: i64) -> Self {
///         Self { number }
///     }
///
///     pub fn as_i64(&self) -> &i64 {
///        &self.0
///     }
/// }
///
/// diesel_i64_wrapper!(NumberWrapper);
///
/// ```
#[macro_export]
macro_rules! diesel_i64_wrapper {
    ($name:ty) => {
        impl<DB: diesel::backend::Backend>
            diesel::deserialize::FromSql<diesel::sql_types::BigInt, DB> for $name
        where
            i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, DB>,
        {
            fn from_sql(
                value: <DB as diesel::backend::Backend>::RawValue<'_>,
            ) -> diesel::deserialize::Result<Self> {
                let value = i64::from_sql(value)?;
                Ok(<$name>::new(value))
            }
        }

        impl<DB: diesel::backend::Backend> diesel::serialize::ToSql<diesel::sql_types::BigInt, DB>
            for $name
        where
            i64: diesel::serialize::ToSql<diesel::sql_types::BigInt, DB>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, DB>,
            ) -> diesel::serialize::Result {
                self.as_i64().to_sql(out)
            }
        }
    };
}

pub(crate) use diesel_i64_wrapper;


/// Enum type must have TryFrom implementation.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// #[derive(
///     diesel::FromSqlRow,
///     diesel::AsExpression,
/// )]
/// #[diesel(sql_type = Integer)]
/// pub enum NumberEnum {
///     Value = 1,
/// }
///
/// impl TryFrom<i64> for NumberEnum {
///     type Error = String;
///
///     fn try_from(value: i64) -> Result<Self, Self::Error> {
///         let number_type = match value {
///             0 => Self::Value,
///             value => return Err(format!("Unknown value {}", value)),
///         };
///
///         Ok(number_type)
///     }
/// }
///
/// diesel_i64_try_from!(NumberEnum);
///
/// ```
#[macro_export]
macro_rules! diesel_i64_try_from {
    ($name:ty) => {
        impl<DB: diesel::backend::Backend>
            diesel::deserialize::FromSql<diesel::sql_types::BigInt, DB> for $name
        where
            i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, DB>,
        {
            fn from_sql(
                value: <DB as diesel::backend::Backend>::RawValue<'_>,
            ) -> diesel::deserialize::Result<Self> {
                let value = i64::from_sql(value)?;
                TryInto::<$name>::try_into(value)
                    .map_err(|e| e.into())
            }
        }

        // TODO: Support other databases?
        // https://docs.diesel.rs/2.0.x/diesel/serialize/trait.ToSql.html

        impl diesel::serialize::ToSql<diesel::sql_types::BigInt, diesel::sqlite::Sqlite>
            for NextQueueNumberType
        where
            i64: diesel::serialize::ToSql<diesel::sql_types::BigInt, diesel::sqlite::Sqlite>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, diesel::sqlite::Sqlite>,
            ) -> diesel::serialize::Result {
                let value = *self as i64;
                out.set_value(value);
                Ok(diesel::serialize::IsNull::No)
            }
        }
    };
}

pub(crate) use diesel_i64_try_from;
