use config::Config;
use model::UnixTime;
use model_server_data::LimitedActionStatus;
use simple_backend_utils::time::next_possible_utc_date_time_value;

#[derive(Debug, Default)]
pub struct ChatLimits {
    pub like_limit: AutoResetLimit<DailyLimit>,
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
pub struct AutoResetLimit<R: ResetLogic> {
    value: u8,
    reset_provider: R,
}

impl<R: ResetLogic> AutoResetLimit<R> {
    pub fn is_limit_not_reached(&mut self, config: &Config) -> bool {
        if let Some(count_left) = self.count_left_mut(config) {
            count_left > 0
        } else {
            true
        }
    }

    pub fn increment_if_possible(&mut self, config: &Config) -> LimitStatus {
        let Some(max_value) = self.reset_provider.max_value(config) else {
            return LimitStatus::Ok;
        };

        if self.reset_provider.reset_can_be_done_and_reset(config) {
            self.value = 0;
        }

        if self.value >= max_value {
            LimitStatus::IncrementingFailed
        } else {
            self.value += 1;
            if self.value >= max_value {
                LimitStatus::LimitReached
            } else {
                LimitStatus::Ok
            }
        }
    }

    /// Returns None if limit is disabled.
    pub fn count_left_mut(&mut self, config: &Config) -> Option<u8> {
        let max_value = self.reset_provider.max_value(config)?;

        if self.reset_provider.reset_can_be_done_and_reset(config) {
            self.value = 0;
        }

        Some(max_value.saturating_sub(self.value))
    }

    pub fn count_left(&self, config: &Config) -> Option<u8> {
        let max_value = self.reset_provider.max_value(config)?;
        if self.reset_provider.reset_can_be_done(config) {
            Some(max_value)
        } else {
            Some(max_value.saturating_sub(self.value))
        }
    }
}

pub trait ResetLogic: Default {
    fn reset_can_be_done_and_reset(&mut self, config: &Config) -> bool;
    fn reset_can_be_done(&self, config: &Config) -> bool;
    /// If None the limit is not enabled.
    fn max_value(&self, config: &Config) -> Option<u8>;
}

#[derive(Debug, Default)]
pub struct DailyLimit {
    next_reset: Option<UnixTime>,
}

impl DailyLimit {
    fn reset_can_be_done_internal(&self, config: &Config) -> Option<chrono::DateTime<chrono::Utc>> {
        let reset_time = config.client_features().and_then(|v| v.limits.likes.like_sending.as_ref()).map(|v| v.reset_time)?;

        let current_time = chrono::Utc::now();

        if let Some(next_reset) = self.next_reset {
            if Into::<UnixTime>::into(current_time).ut < next_reset.ut {
                return None;
            }
        }

        let Ok(next_reset) = next_possible_utc_date_time_value(current_time, reset_time) else {
            return None;
        };

        Some(next_reset)
    }
}

impl ResetLogic for DailyLimit {
    fn reset_can_be_done_and_reset(&mut self, config: &Config) -> bool {
        if let Some(next_reset) = self.reset_can_be_done_internal(config) {
            self.next_reset = Some(next_reset.into());
            true
        } else {
            false
        }
    }

    fn reset_can_be_done(&self, config: &Config) -> bool {
        self.reset_can_be_done_internal(config).is_some()
    }

    fn max_value(&self, config: &Config) -> Option<u8> {
        config.client_features().and_then(|v| v.limits.likes.like_sending.as_ref()).map(|v| v.daily_limit)
    }
}
