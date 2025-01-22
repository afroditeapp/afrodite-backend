use std::time::Duration;

use chrono::{NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

#[derive(thiserror::Error, Debug)]
pub enum SleepUntilClockIsAtError {
    #[error("Target time value is invalid")]
    TargetTimeValueInvalid,
    #[error("Creating todays' target date time failed")]
    DateTimeCreationForTodayFailed,
    #[error("Creating tomorrow's target date time failed")]
    DateTimeCreationForTomorrowFailed,
}

pub async fn sleep_until_current_time_is_at(
    wanted_time: UtcTimeValue,
) -> Result<(), SleepUntilClockIsAtError> {
    let duration_seconds = Duration::from_secs(seconds_until_current_time_is_at(wanted_time)?);
    sleep(duration_seconds).await;
    Ok(())
}

pub fn seconds_until_current_time_is_at(
    wanted_time: UtcTimeValue,
) -> Result<u64, SleepUntilClockIsAtError> {
    let now: chrono::DateTime<Utc> = Utc::now();

    let target_time =
        NaiveTime::from_hms_opt(wanted_time.0.hours.into(), wanted_time.0.minutes.into(), 0)
            .ok_or(SleepUntilClockIsAtError::TargetTimeValueInvalid)?;

    let target_date_time = now
        .with_time(target_time)
        .single()
        .ok_or(SleepUntilClockIsAtError::DateTimeCreationForTodayFailed)?;

    let duration = if target_date_time > now {
        target_date_time - now
    } else {
        let tomorrow = now + Duration::from_secs(24 * 60 * 60);
        let tomorrow_target_date_time = tomorrow
            .with_time(target_time)
            .single()
            .ok_or(SleepUntilClockIsAtError::DateTimeCreationForTomorrowFailed)?;
        tomorrow_target_date_time - now
    };

    Ok(duration.abs().num_seconds() as u64)
}

/// UTC time value
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct UtcTimeValue(pub TimeValue);

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(try_from = "String")]
pub struct TimeValue {
    pub hours: u8,
    pub minutes: u8,
}

impl TimeValue {
    const MAX_HOURS: u8 = 23;
    const MAX_MINUTES: u8 = 59;

    /// Panics if values are out of range
    pub const fn new(hours: u8, minutes: u8) -> Self {
        if hours > Self::MAX_HOURS {
            panic!("Hours value is not valid");
        }

        if minutes > Self::MAX_MINUTES {
            panic!("Minutes value is not valid");
        }

        Self { hours, minutes }
    }
}

impl TryFrom<String> for TimeValue {
    type Error = String;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let iter = value.trim().split(':');
        let values: Vec<&str> = iter.collect();
        match values[..] {
            [hours, minutes] => {
                let hours: u8 = hours
                    .parse()
                    .map_err(|e: std::num::ParseIntError| e.to_string())?;
                if hours > Self::MAX_HOURS {
                    return Err(format!(
                        "Max value for hours is {}, current value: {hours}",
                        Self::MAX_HOURS
                    ));
                }
                let minutes: u8 = minutes
                    .parse()
                    .map_err(|e: std::num::ParseIntError| e.to_string())?;
                if minutes > Self::MAX_MINUTES {
                    return Err(format!(
                        "Max value for minutes is {}, current value: {minutes}",
                        Self::MAX_MINUTES
                    ));
                }
                Ok(TimeValue { hours, minutes })
            }
            _ => Err(format!("Unknown values: {:?}", values)),
        }
    }
}

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
