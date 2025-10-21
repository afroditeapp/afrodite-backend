/// Type must have new() and to_uuid() methods. Also diesel::FromSqlRow and
/// diesel::AsExpression derives are needed.
///
/// ```
/// use diesel::sql_types::Binary;
/// use simple_backend_model::diesel_uuid_wrapper;
/// use simple_backend_utils::UuidBase64Url;
///
/// #[derive(
///     Debug,
///     diesel::FromSqlRow,
///     diesel::AsExpression,
/// )]
/// #[diesel(sql_type = Binary)]
/// pub struct UuidWrapper {
///     uuid: UuidBase64Url,
/// }
///
/// impl UuidWrapper {
///     pub fn diesel_uuid_wrapper_new(uuid: UuidBase64Url) -> Self {
///         Self { uuid }
///     }
///
///     pub fn diesel_uuid_wrapper_as_uuid(&self) -> &UuidBase64Url {
///         &self.uuid
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
                let uuid = simple_backend_utils::UuidBase64Url::new(uuid);
                Ok(<$name>::diesel_uuid_wrapper_new(uuid))
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
                let uuid = self.diesel_uuid_wrapper_as_uuid().as_uuid();
                let bytes: &[u8] = uuid.as_bytes();
                bytes.to_sql(out)
            }
        }
    };
}

/// Type must have new() and as_str() methods.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// use diesel::sql_types::Text;
/// use simple_backend_model::diesel_string_wrapper;
///
/// #[derive(
///     Debug,
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

/// Type must have new() and as_str() methods.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// use diesel::sql_types::Text;
/// use simple_backend_model::diesel_non_empty_string_wrapper;
/// use simple_backend_model::NonEmptyString;
///
/// #[derive(
///     Debug,
///     diesel::FromSqlRow,
///     diesel::AsExpression,
/// )]
/// #[diesel(sql_type = Text)]
/// pub struct NonEmptyStringWrapper {
///     value: NonEmptyString,
/// }
///
/// impl NonEmptyStringWrapper {
///     pub fn new(value: NonEmptyString) -> Self {
///         Self { value }
///     }
///
///     pub fn as_str(&self) -> &str {
///        &self.text
///     }
/// }
///
/// diesel_non_empty_string_wrapper!(NonEmptyStringWrapper);
/// ```
#[macro_export]
macro_rules! diesel_non_empty_string_wrapper {
    ($name:ty) => {
        impl<DB: diesel::backend::Backend> diesel::deserialize::FromSql<diesel::sql_types::Text, DB>
            for $name
        where
            String: diesel::deserialize::FromSql<diesel::sql_types::Text, DB>,
        {
            fn from_sql(
                value: <DB as diesel::backend::Backend>::RawValue<'_>,
            ) -> diesel::deserialize::Result<Self> {
                let value = NonEmptyString::from_sql(value)?;
                Ok(<$name>::new(value))
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

/// Type must have new() and as_i64() methods.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// use diesel::sql_types::Integer;
/// use simple_backend_model::diesel_i64_wrapper;
///
/// #[derive(
///     Debug,
///     Clone,
///     Copy,
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
///        &self.number
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

/// The struct needs to have `TryFrom<i64>` and `Into<i64>` implementations.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// use diesel::sql_types::Integer;
/// use simple_backend_model::diesel_i64_struct_try_from;
///
/// #[derive(
///     Debug,
///     Clone,
///     Copy,
///     diesel::FromSqlRow,
///     diesel::AsExpression,
/// )]
/// #[diesel(sql_type = Integer)]
/// pub struct NumberStruct {
///     value: i64,
/// }
///
/// impl TryFrom<i64> for NumberStruct {
///     type Error = String;
///
///     fn try_from(value: i64) -> Result<Self, Self::Error> {
///         Ok(NumberStruct { value: value })
///     }
/// }
///
/// impl From<NumberStruct> for i64 {
///     fn from(value: NumberStruct) -> Self {
///         value.value
///     }
/// }
///
/// diesel_i64_struct_try_from!(NumberStruct);
///
/// ```
#[macro_export]
macro_rules! diesel_i64_struct_try_from {
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
                TryInto::<$name>::try_into(value).map_err(|e| e.into())
            }
        }

        // TODO(future): Support other databases?
        // https://docs.diesel.rs/2.0.x/diesel/serialize/trait.ToSql.html

        impl diesel::serialize::ToSql<diesel::sql_types::BigInt, diesel::sqlite::Sqlite> for $name
        where
            i64: diesel::serialize::ToSql<diesel::sql_types::BigInt, diesel::sqlite::Sqlite>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, diesel::sqlite::Sqlite>,
            ) -> diesel::serialize::Result {
                let value = Into::<i64>::into(*self);
                out.set_value(value);
                Ok(diesel::serialize::IsNull::No)
            }
        }
    };
}

/// Version of diesel_i64_struct_try_from! for bytes.
/// The struct or enum needs to have `AsRef<&[u8]>` implementation.
#[macro_export]
macro_rules! diesel_bytes_try_from {
    ($name:ty) => {
        impl<DB: diesel::backend::Backend>
            diesel::deserialize::FromSql<diesel::sql_types::Binary, DB> for $name
        where
            Vec<u8>: diesel::deserialize::FromSql<diesel::sql_types::Binary, DB>,
        {
            fn from_sql(
                value: <DB as diesel::backend::Backend>::RawValue<'_>,
            ) -> diesel::deserialize::Result<Self> {
                let value = Vec::<u8>::from_sql(value)?;
                TryInto::<$name>::try_into(value.as_slice()).map_err(|e| e.into())
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
                let value = AsRef::<[u8]>::as_ref(self);
                value.to_sql(out)
            }
        }
    };
}
