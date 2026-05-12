use serde::{Deserialize, Serialize};

use crate::consts::{GIB_IN_BYTES, KIB_IN_BYTES, MIB_IN_BYTES};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct ByteCount {
    /// Keep this as i64 because Bash doesn't support u64
    bytes: i64,
}

impl ByteCount {
    pub fn bytes(&self) -> i64 {
        self.bytes
    }

    pub fn from_megabytes(mb: u32) -> Self {
        Self {
            bytes: Into::<i64>::into(mb) * MIB_IN_BYTES as i64,
        }
    }
}

impl TryFrom<String> for ByteCount {
    type Error = String;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let input = value.trim();
        if input.is_empty() {
            return Err("Byte count cannot be empty".to_string());
        }

        // Check if the input ends with a unit suffix
        let (number_str, unit) = if input.ends_with('K') || input.ends_with('k') {
            (&input[..input.len() - 1], "K")
        } else if input.ends_with('M') || input.ends_with('m') {
            (&input[..input.len() - 1], "M")
        } else if input.ends_with('G') || input.ends_with('g') {
            (&input[..input.len() - 1], "G")
        } else {
            (input, "B")
        };

        let number: u32 = number_str
            .parse()
            .map_err(|e: std::num::ParseIntError| {
                format!("Parsing byte count failed: {e}, current value: {input}, example values: 100K, 50M, 1G")
            })?;

        let number = Into::<i64>::into(number);

        let bytes = match unit {
            "B" => number,
            "K" => number * KIB_IN_BYTES as i64,
            "M" => number * MIB_IN_BYTES as i64,
            "G" => number * GIB_IN_BYTES as i64,
            _ => unreachable!(),
        };

        Ok(ByteCount { bytes })
    }
}

impl From<ByteCount> for String {
    fn from(value: ByteCount) -> Self {
        value.bytes.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::ByteCount;
    use crate::consts::{GIB_IN_BYTES, KIB_IN_BYTES, MIB_IN_BYTES};

    #[test]
    fn byte_count_parses_bytes_without_unit() {
        let parsed = ByteCount::try_from("123".to_string()).unwrap();
        assert_eq!(parsed.bytes(), 123);
    }

    #[test]
    fn byte_count_parses_kibibytes() {
        let parsed_upper = ByteCount::try_from("2K".to_string()).unwrap();
        assert_eq!(parsed_upper.bytes(), 2 * KIB_IN_BYTES as i64);

        let parsed_lower = ByteCount::try_from("2k".to_string()).unwrap();
        assert_eq!(parsed_lower.bytes(), 2 * KIB_IN_BYTES as i64);
    }

    #[test]
    fn byte_count_parses_mebibytes() {
        let parsed_upper = ByteCount::try_from("3M".to_string()).unwrap();
        assert_eq!(parsed_upper.bytes(), 3 * MIB_IN_BYTES as i64);

        let parsed_lower = ByteCount::try_from("3m".to_string()).unwrap();
        assert_eq!(parsed_lower.bytes(), 3 * MIB_IN_BYTES as i64);
    }

    #[test]
    fn byte_count_parses_gibibytes() {
        let parsed_upper = ByteCount::try_from("4G".to_string()).unwrap();
        assert_eq!(parsed_upper.bytes(), 4 * GIB_IN_BYTES as i64);

        let parsed_lower = ByteCount::try_from("4g".to_string()).unwrap();
        assert_eq!(parsed_lower.bytes(), 4 * GIB_IN_BYTES as i64);
    }

    #[test]
    fn byte_count_rejects_negative_without_unit() {
        assert!(ByteCount::try_from("-1".to_string()).is_err());
    }

    #[test]
    fn byte_count_rejects_negative_with_unit() {
        assert!(ByteCount::try_from("-1M".to_string()).is_err());
    }
}
