use diesel::sql_types::Text;
use serde::{Deserialize, Deserializer, Serialize, de::Error};
use utoipa::ToSchema;

/// A string wrapper that ensures the string is not empty.
/// This type is used for TEXT columns that should not allow empty strings.
/// In the database, these columns are NULL when there is no value, and this
/// type represents non-NULL values that must be non-empty.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, ToSchema, diesel::FromSqlRow, diesel::AsExpression,
)]
#[diesel(sql_type = Text)]
#[serde(transparent)]
#[schema(value_type = String)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    /// Create a new NonEmptyString with validation. Returns None if the string is empty.
    pub fn from_string(text: String) -> Option<Self> {
        if text.is_empty() {
            None
        } else {
            Some(Self(text))
        }
    }

    /// Get the string as a &str.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into a String.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<DB: diesel::backend::Backend> diesel::deserialize::FromSql<diesel::sql_types::Text, DB>
    for NonEmptyString
where
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(
        value: <DB as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let string = String::from_sql(value)?;
        if string.is_empty() {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "NonEmptyString cannot be empty",
            )))
        } else {
            Ok(NonEmptyString(string))
        }
    }
}

impl<DB: diesel::backend::Backend> diesel::serialize::ToSql<diesel::sql_types::Text, DB>
    for NonEmptyString
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

impl<'de> Deserialize<'de> for NonEmptyString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_string(s).ok_or_else(|| Error::custom("NonEmptyString cannot be empty"))
    }
}

#[cfg(test)]
mod tests {
    use serde_json;

    use super::*;

    #[test]
    fn test_non_empty_string_creation() {
        assert!(NonEmptyString::from_string("test".to_string()).is_some());
        assert!(NonEmptyString::from_string("".to_string()).is_none());
    }

    #[test]
    fn test_non_empty_string_methods() {
        let s = NonEmptyString::from_string("test".to_string()).unwrap();
        assert_eq!(s.as_str(), "test");
        assert_eq!(s.into_string(), "test");
    }

    #[test]
    fn test_deserialize_success() {
        let json = "\"test\"";
        let result: NonEmptyString = serde_json::from_str(json).unwrap();
        assert_eq!(result.as_str(), "test");
    }

    #[test]
    fn test_deserialize_failure_empty() {
        let json = "\"\"";
        let result: Result<NonEmptyString, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
