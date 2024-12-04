use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(try_from = "String")]
pub struct DurationValue {
    pub seconds: u32,
}

impl DurationValue {
    pub const fn from_days(days: u32) -> Self {
        Self { seconds: days * 60 * 60 * 24 }
    }
}

impl TryFrom<String> for DurationValue {
    type Error = String;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let input = value.trim();
        if input.len() < 2 {
            return Err(format!(
                "Parsing duration failed, current value: {}, example value: 1s",
                input
            ));
        }
        let Some((number, time_unit)) = input.split_at_checked(input.len() - 1) else {
            return Err(format!(
                "Parsing duration failed, current value: {}, example value: 1s",
                input
            ));
        };
        let number: u32 = number
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?;
        let seconds = match time_unit {
            "s" => number,
            "m" => number * 60,
            "h" => number * 60 * 60,
            "d" => number * 60 * 60 * 24,
            time_unit => return Err(format!("Unknown time unit: {}, supported units: s, m, h, d", time_unit)),
        };

        Ok(DurationValue { seconds })
    }
}
