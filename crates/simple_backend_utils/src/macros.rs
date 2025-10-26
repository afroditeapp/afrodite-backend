/// The struct needs to have `TryFrom<UuidBase64Url>` and `AsRef<UuidBase64Url>` implementations.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// use diesel::sql_types::Binary;
/// use simple_backend_utils::diesel_uuid_wrapper;
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
/// impl TryFrom<UuidBase64Url> for UuidWrapper {
///     type Error = String;
///
///     fn try_from(uuid: UuidBase64Url) -> Result<Self, Self::Error> {
///         Ok(Self { uuid })
///     }
/// }
///
/// impl AsRef<UuidBase64Url> for UuidWrapper {
///     fn as_ref(&self) -> &UuidBase64Url {
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
                TryInto::<$name>::try_into(uuid).map_err(|e| e.into())
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
                let uuid = AsRef::<simple_backend_utils::UuidBase64Url>::as_ref(self).as_uuid();
                let bytes: &[u8] = uuid.as_bytes();
                bytes.to_sql(out)
            }
        }
    };
}

/// The struct needs to have `TryFrom<String>` and `AsRef<str>` implementations.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// use diesel::sql_types::Text;
/// use simple_backend_utils::diesel_string_wrapper;
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
/// impl TryFrom<String> for StringWrapper {
///     type Error = String;
///
///     fn try_from(text: String) -> Result<Self, Self::Error> {
///         Ok(Self { text })
///     }
/// }
///
/// impl AsRef<str> for StringWrapper {
///     fn as_ref(&self) -> &str {
///         &self.text
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
                TryInto::<$name>::try_into(string).map_err(|e| e.into())
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
                AsRef::<str>::as_ref(self).to_sql(out)
            }
        }
    };
}

/// The struct needs to have `TryFrom<NonEmptyString>` and `AsRef<str>` implementations.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// use diesel::sql_types::Text;
/// use simple_backend_utils::diesel_non_empty_string_wrapper;
/// use simple_backend_utils::string::NonEmptyString;
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
/// impl TryFrom<NonEmptyString> for NonEmptyStringWrapper {
///     type Error = String;
///
///     fn try_from(value: NonEmptyString) -> Result<Self, Self::Error> {
///         Ok(Self { value })
///     }
/// }
///
/// impl AsRef<str> for NonEmptyStringWrapper {
///     fn as_ref(&self) -> &str {
///         &self.value.as_str()
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
                TryInto::<$name>::try_into(value).map_err(|e| e.into())
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
                AsRef::<str>::as_ref(self).to_sql(out)
            }
        }
    };
}

/// The struct needs to have `TryFrom<i64>` and `AsRef<i64>` implementations.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// use diesel::sql_types::BigInt;
/// use simple_backend_utils::diesel_i64_wrapper;
///
/// #[derive(
///     Debug,
///     Clone,
///     Copy,
///     diesel::FromSqlRow,
///     diesel::AsExpression,
/// )]
/// #[diesel(sql_type = BigInt)]
/// pub struct NumberWrapper {
///     number: i64,
/// }
///
/// impl TryFrom<i64> for NumberWrapper {
///     type Error = String;
///
///     fn try_from(number: i64) -> Result<Self, Self::Error> {
///         Ok(Self { number })
///     }
/// }
///
/// impl AsRef<i64> for NumberWrapper {
///     fn as_ref(&self) -> &i64 {
///         &self.number
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
                TryInto::<$name>::try_into(value).map_err(|e| e.into())
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
                AsRef::<i64>::as_ref(self).to_sql(out)
            }
        }
    };
}

/// The struct needs to have `TryFrom<i32>` and `AsRef<i32>` implementations.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// use diesel::sql_types::Integer;
/// use simple_backend_utils::diesel_i32_wrapper;
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
///     value: i32,
/// }
///
/// impl TryFrom<i32> for NumberStruct {
///     type Error = String;
///
///     fn try_from(value: i32) -> Result<Self, Self::Error> {
///         Ok(NumberStruct { value })
///     }
/// }
///
/// impl AsRef<i32> for NumberStruct {
///     fn as_ref(&self) -> &i32 {
///         &self.value
///     }
/// }
///
/// diesel_i32_wrapper!(NumberStruct);
///
/// ```
#[macro_export]
macro_rules! diesel_i32_wrapper {
    ($name:ty) => {
        impl<DB: diesel::backend::Backend>
            diesel::deserialize::FromSql<diesel::sql_types::Integer, DB> for $name
        where
            i32: diesel::deserialize::FromSql<diesel::sql_types::Integer, DB>,
        {
            fn from_sql(
                value: <DB as diesel::backend::Backend>::RawValue<'_>,
            ) -> diesel::deserialize::Result<Self> {
                let value = i32::from_sql(value)?;
                TryInto::<$name>::try_into(value).map_err(|e| e.into())
            }
        }

        impl<DB: diesel::backend::Backend> diesel::serialize::ToSql<diesel::sql_types::Integer, DB>
            for $name
        where
            i32: diesel::serialize::ToSql<diesel::sql_types::Integer, DB>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, DB>,
            ) -> diesel::serialize::Result {
                let value = AsRef::<i32>::as_ref(self);
                value.to_sql(out)
            }
        }
    };
}

/// The struct needs to have `TryFrom<i16>` and `AsRef<i16>` implementations.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// use diesel::sql_types::SmallInt;
/// use simple_backend_utils::diesel_i16_wrapper;
///
/// #[derive(
///     Debug,
///     Clone,
///     Copy,
///     diesel::FromSqlRow,
///     diesel::AsExpression,
/// )]
/// #[diesel(sql_type = SmallInt)]
/// pub struct SmallNumberWrapper {
///     number: i16,
/// }
///
/// impl TryFrom<i16> for SmallNumberWrapper {
///     type Error = String;
///
///     fn try_from(number: i16) -> Result<Self, Self::Error> {
///         Ok(Self { number })
///     }
/// }
///
/// impl AsRef<i16> for SmallNumberWrapper {
///     fn as_ref(&self) -> &i16 {
///         &self.number
///     }
/// }
///
/// diesel_i16_wrapper!(SmallNumberWrapper);
///
/// ```
#[macro_export]
macro_rules! diesel_i16_wrapper {
    ($name:ty) => {
        impl<DB: diesel::backend::Backend>
            diesel::deserialize::FromSql<diesel::sql_types::SmallInt, DB> for $name
        where
            i16: diesel::deserialize::FromSql<diesel::sql_types::SmallInt, DB>,
        {
            fn from_sql(
                value: <DB as diesel::backend::Backend>::RawValue<'_>,
            ) -> diesel::deserialize::Result<Self> {
                let value = i16::from_sql(value)?;
                TryInto::<$name>::try_into(value).map_err(|e| e.into())
            }
        }

        impl<DB: diesel::backend::Backend> diesel::serialize::ToSql<diesel::sql_types::SmallInt, DB>
            for $name
        where
            i16: diesel::serialize::ToSql<diesel::sql_types::SmallInt, DB>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, DB>,
            ) -> diesel::serialize::Result {
                AsRef::<i16>::as_ref(self).to_sql(out)
            }
        }
    };
}

/// The struct needs to have `Into<i16>` and `TryFrom<i16>` implementations.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// use diesel::sql_types::SmallInt;
/// use simple_backend_utils::diesel_db_i16_is_i8_struct;
///
/// #[derive(
///     Debug,
///     Clone,
///     Copy,
///     diesel::FromSqlRow,
///     diesel::AsExpression,
/// )]
/// #[diesel(sql_type = SmallInt)]
/// pub struct I8Struct {
///     value: i8,
/// }
///
/// impl From<I8Struct> for i16 {
///     fn from(value: I8Struct) -> Self {
///         value.value.into()
///     }
/// }
///
/// impl TryFrom<i16> for I8Struct {
///     type Error = String;
///
///     fn try_from(value: i16) -> Result<Self, Self::Error> {
///         let value = i8::try_from(value).map_err(|e| e.to_string())?;
///         Ok(I8Struct { value })
///     }
/// }
///
/// diesel_db_i16_is_i8_struct!(I8Struct);
///
/// ```
#[macro_export]
macro_rules! diesel_db_i16_is_i8_struct {
    ($name:ty) => {
        impl<DB: diesel::backend::Backend>
            diesel::deserialize::FromSql<diesel::sql_types::SmallInt, DB> for $name
        where
            i16: diesel::deserialize::FromSql<diesel::sql_types::SmallInt, DB>,
        {
            fn from_sql(
                value: <DB as diesel::backend::Backend>::RawValue<'_>,
            ) -> diesel::deserialize::Result<Self> {
                let value = i16::from_sql(value)?;
                TryInto::<$name>::try_into(value).map_err(|e| e.into())
            }
        }

        impl diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::pg::Pg> for $name
        where
            i16: diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::pg::Pg>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
            ) -> diesel::serialize::Result {
                let value: i16 = Into::<i16>::into(*self).into();
                <i16 as diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::pg::Pg>>::to_sql(&value, &mut out.reborrow())
            }
        }

        impl diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::sqlite::Sqlite> for $name
        where
            i16: diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::sqlite::Sqlite>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, diesel::sqlite::Sqlite>,
            ) -> diesel::serialize::Result {
                let value: i32 = Into::<i16>::into(*self).into();
                out.set_value(value);
                Ok(diesel::serialize::IsNull::No)
            }
        }

        impl diesel::serialize::ToSql<diesel::sql_types::SmallInt, $crate::db::MultiBackend> for $name
        where
            i16: diesel::serialize::ToSql<diesel::sql_types::SmallInt, $crate::db::MultiBackend>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, $crate::db::MultiBackend>,
            ) -> diesel::serialize::Result {
                out.set_value((diesel::sql_types::SmallInt, self));
                Ok(diesel::serialize::IsNull::No)
            }
        }
    };
}

/// The struct needs to have `Into<i16>` and `TryFrom<i16>` implementations.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
///
/// ```
/// use diesel::sql_types::SmallInt;
/// use simple_backend_utils::diesel_db_i16_is_u8_struct;
///
/// #[derive(
///     Debug,
///     Clone,
///     Copy,
///     diesel::FromSqlRow,
///     diesel::AsExpression,
/// )]
/// #[diesel(sql_type = SmallInt)]
/// pub struct U8Struct {
///     value: u8,
/// }
///
/// impl From<U8Struct> for i16 {
///     fn from(value: U8Struct) -> Self {
///         value.value.into()
///     }
/// }
///
/// impl TryFrom<i16> for U8Struct {
///     type Error = String;
///
///     fn try_from(value: i16) -> Result<Self, Self::Error> {
///         let value = u8::try_from(value).map_err(|e| e.to_string())?;
///         Ok(U8Struct { value })
///     }
/// }
///
/// diesel_db_i16_is_u8_struct!(U8Struct);
///
/// ```
#[macro_export]
macro_rules! diesel_db_i16_is_u8_struct {
    ($name:ty) => {
        impl<DB: diesel::backend::Backend>
            diesel::deserialize::FromSql<diesel::sql_types::SmallInt, DB> for $name
        where
            i16: diesel::deserialize::FromSql<diesel::sql_types::SmallInt, DB>,
        {
            fn from_sql(
                value: <DB as diesel::backend::Backend>::RawValue<'_>,
            ) -> diesel::deserialize::Result<Self> {
                let value = i16::from_sql(value)?;
                TryInto::<$name>::try_into(value).map_err(|e| e.into())
            }
        }

        impl diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::pg::Pg> for $name
        where
            i16: diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::pg::Pg>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
            ) -> diesel::serialize::Result {
                let value: i16 = Into::<i16>::into(*self).into();
                <i16 as diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::pg::Pg>>::to_sql(&value, &mut out.reborrow())
            }
        }

        impl diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::sqlite::Sqlite> for $name
        where
            i16: diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::sqlite::Sqlite>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, diesel::sqlite::Sqlite>,
            ) -> diesel::serialize::Result {
                let value: i32 = Into::<i16>::into(*self).into();
                out.set_value(value);
                Ok(diesel::serialize::IsNull::No)
            }
        }

        impl diesel::serialize::ToSql<diesel::sql_types::SmallInt, $crate::db::MultiBackend> for $name
        where
            i16: diesel::serialize::ToSql<diesel::sql_types::SmallInt, $crate::db::MultiBackend>,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, $crate::db::MultiBackend>,
            ) -> diesel::serialize::Result {
                out.set_value((diesel::sql_types::SmallInt, self));
                Ok(diesel::serialize::IsNull::No)
            }
        }
    };
}

/// The struct needs to have `TryFrom<i64>` and `AsRef<&[u8]>` implementations.
/// Also diesel::FromSqlRow and diesel::AsExpression derives are needed.
#[macro_export]
macro_rules! diesel_bytes_wrapper {
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
