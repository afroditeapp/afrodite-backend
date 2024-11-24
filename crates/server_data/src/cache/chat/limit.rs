use chrono::Datelike;
use config::Config;
use model_chat::LimitedActionStatus;
use server_common::data::cache::CacheError;

use error_stack::Result;

const MAX_VALUE_1: u8 = 1;

#[derive(Debug, Default)]
pub struct ChatLimits {
    pub like_limit: AutoResetLimit<DailyLimit, MAX_VALUE_1>,
}

pub enum LimitStatus {
    /// Incrementing next time is possible.
    Ok,
    /// Next incrementing before reset will fail
    LimitReached,
    /// Limit already reached
    IncrementingFailed,
}

impl LimitStatus {
    pub fn to_action_status(&self) -> LimitedActionStatus {
        match *self {
            Self::Ok => LimitedActionStatus::Success,
            Self::LimitReached => LimitedActionStatus::SuccessAndLimitReached,
            Self::IncrementingFailed => LimitedActionStatus::FailureLimitAlreadyReached,
        }
    }
}

#[derive(Debug, Default)]
pub struct AutoResetLimit<R: ResetLogic, const MAX_VALUE: u8> {
    value: u8,
    reset_provider: R,
}

impl <R: ResetLogic, const MAX_VALUE: u8> AutoResetLimit<R, MAX_VALUE> {
    pub fn is_limit_not_reached(&mut self, config: &Config) -> Result<bool, CacheError> {
        if self.reset_provider.reset_can_be_done(config)? {
            self.value = 0;
        }

        Ok(self.value < MAX_VALUE)
    }

    pub fn increment_if_possible(&mut self, config: &Config) -> Result<LimitStatus, CacheError> {
        if self.reset_provider.reset_can_be_done(config)? {
            self.value = 0;
        }

        if self.value >= MAX_VALUE {
            Ok(LimitStatus::IncrementingFailed)
        } else {
            self.value += 1;
            if self.value >= MAX_VALUE {
                Ok(LimitStatus::LimitReached)
            } else {
                Ok(LimitStatus::Ok)
            }
        }
    }
}

pub trait ResetLogic: Default {
    fn reset_can_be_done(&mut self, config: &Config) -> Result<bool, CacheError>;
}

#[derive(Debug, Default)]
pub struct DailyLimit {
    previous_reset_day: Option<u8>,
}

impl ResetLogic for DailyLimit {
    fn reset_can_be_done(&mut self, config: &Config) -> Result<bool, CacheError> {
        let time = chrono::Utc::now().with_timezone(&config.reset_likes_utc_offset());
        let current_day = time.day() as u8;
        let reset_can_be_done = if let Some(previous_reset_day) = self.previous_reset_day {
            previous_reset_day != current_day
        } else {
            true
        };
        if reset_can_be_done {
            self.previous_reset_day = Some(current_day);
        }
        Ok(reset_can_be_done)
    }
}
