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
            Self::High => "6",
            Self::Medium => "5",
            Self::Low => "4",
        }
    }

    /// Protocol byte value.
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::High => 6,
            Self::Medium => 5,
            Self::Low => 4,
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            6 => Some(Self::High),
            5 => Some(Self::Medium),
            4 => Some(Self::Low),
            _ => None,
        }
    }

    pub const VARIANT_COUNT: usize = 3;

    pub fn all_variants() -> [Self; 3] {
        [Self::High, Self::Medium, Self::Low]
    }
}
