use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

/// ISO 3166-1 alpha-2 country code.
///
/// The code is always stored as uppercase ASCII. Use [`IsoCountryCode::new`]
/// or deserialization to create a value; both normalize the input to
/// uppercase.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize)]
pub struct IsoCountryCode(String);

impl IsoCountryCode {
    /// Creates a new country code, normalizing it to uppercase ASCII.
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into().to_ascii_uppercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl PartialEq<str> for IsoCountryCode {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for IsoCountryCode {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl fmt::Display for IsoCountryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for IsoCountryCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let code = String::deserialize(deserializer)?;
        Ok(Self::new(code))
    }
}
