use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(try_from = "String")]
pub struct VersionNumber {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl PartialOrd for VersionNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VersionNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        let major_cmp = self.major.cmp(&other.major);
        if major_cmp != Ordering::Equal {
            return major_cmp;
        }

        let minor_cmp = self.minor.cmp(&other.minor);
        if minor_cmp != Ordering::Equal {
            return minor_cmp;
        }

        self.patch.cmp(&other.patch)
    }
}

impl TryFrom<String> for VersionNumber {
    type Error = String;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let mut numbers = value.split('.');
        let error = || format!("Version {value} is not formatted like 1.0.0");
        let major_str = numbers.next().ok_or_else(error)?;
        let minor_str = numbers.next().ok_or_else(error)?;
        let patch_str = numbers.next().ok_or_else(error)?;

        let major = major_str.parse::<u16>().map_err(|e| e.to_string())?;
        let minor = minor_str.parse::<u16>().map_err(|e| e.to_string())?;
        let patch = patch_str.parse::<u16>().map_err(|e| e.to_string())?;

        Ok(VersionNumber {
            major,
            minor,
            patch,
        })
    }
}
