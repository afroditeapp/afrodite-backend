use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ContentQualityVariant {
    High,
    Medium,
    Low,
}

impl ContentQualityVariant {
    pub fn variant_suffix(&self) -> &'static str {
        match self {
            Self::High => "_h",
            Self::Medium => "_m",
            Self::Low => "_l",
        }
    }

    /// Protocol byte value matching bitflag positions in ContentQuery.
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::High => 1,
            Self::Medium => 2,
            Self::Low => 4,
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::High),
            2 => Some(Self::Medium),
            4 => Some(Self::Low),
            _ => None,
        }
    }
}
