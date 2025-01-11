use std::time::Duration;

use chrono::{NaiveTime, Utc};
use simple_backend_config::file::UtcTimeValue;
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
    let duration_seconds = Duration::from_secs(duration.abs().num_seconds() as u64);
    sleep(duration_seconds).await;

    Ok(())
}
