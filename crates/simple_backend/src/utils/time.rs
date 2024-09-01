use std::time::Duration;

use chrono::{Local, NaiveTime};
use simple_backend_config::file::LocalTimeValue;
use tokio::time::sleep;

#[derive(thiserror::Error, Debug)]
pub enum SleepUntilLocalClockIsAtError {
    #[error("Local time value is invalid")]
    LocalTimeValueInvalid,
    #[error("Creating todays' target date time failed")]
    DateTimeCreationForTodayFailed,
    #[error("Creating tomorrow's target date time failed")]
    DateTimeCreationForTomorrowFailed,
}

pub async fn sleep_until_local_time_clock_is_at(wanted_time: LocalTimeValue) -> Result<(), SleepUntilLocalClockIsAtError> {
    let now = Local::now();

    let target_time =
        NaiveTime::from_hms_opt(wanted_time.0.hours.into(), wanted_time.0.minutes.into(), 0)
            .ok_or(SleepUntilLocalClockIsAtError::LocalTimeValueInvalid)?;

    let target_date_time = now.with_time(target_time)
        .single()
        .ok_or(SleepUntilLocalClockIsAtError::DateTimeCreationForTodayFailed)?;

    let duration = if target_date_time > now {
        target_date_time - now
    } else {
        let tomorrow = now + Duration::from_secs(24 * 60 * 60);
        let tomorrow_target_date_time = tomorrow.with_time(target_time)
            .single()
            .ok_or(SleepUntilLocalClockIsAtError::DateTimeCreationForTomorrowFailed)?;
        tomorrow_target_date_time - now
    };
    let duration_seconds = Duration::from_secs(duration.abs().num_seconds() as u64);
    sleep(duration_seconds).await;

    Ok(())
}
