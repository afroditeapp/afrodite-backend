use config::Config;
use model::{AccountId, AccountIdInternal};
use server_common::result::Result;
use server_data::{
    cache::{DatabaseCache, api_limits::AllApiLimits},
    result::{WrappedContextExt, WrappedResultExt},
};

/// API routes convert this error to
/// [crate::utils::StatusCode::TOO_MANY_REQUESTS].
#[derive(thiserror::Error, Debug)]
pub enum ApiLimitError {
    #[error("Limit reached error")]
    LimitReached,
    #[error("Cache error")]
    Cache,
    #[error("Resetting API limits failed")]
    ResetFailed,
}

pub struct ApiLimits<'a> {
    cache: &'a DatabaseCache,
    config: &'a Config,
    account_id: AccountId,
}

impl<'a> ApiLimits<'a> {
    pub(crate) fn new(
        cache: &'a DatabaseCache,
        config: &'a Config,
        account_id: AccountIdInternal,
    ) -> Self {
        Self {
            cache,
            config,
            account_id: account_id.into(),
        }
    }

    pub fn profile(self) -> ProfileApiLimits<'a> {
        ProfileApiLimits { limits: self }
    }

    pub fn media(self) -> MediaApiLimits<'a> {
        MediaApiLimits { limits: self }
    }

    pub async fn reset_limits(self) -> Result<(), ApiLimitError> {
        self.cache
            .write_cache_common(self.account_id, |e| {
                e.reset_api_limits();
                Ok(())
            })
            .await
            .change_context(ApiLimitError::ResetFailed)
    }
}

pub struct ProfileApiLimits<'a> {
    limits: ApiLimits<'a>,
}

impl ProfileApiLimits<'_> {
    async fn check(
        &self,
        check: impl FnOnce(&mut AllApiLimits, &Config) -> bool,
    ) -> Result<(), ApiLimitError> {
        let is_limit_reached = self
            .limits
            .cache
            .write_cache_common(self.limits.account_id, |e| {
                Ok(check(e.api_limits(), self.limits.config))
            })
            .await
            .change_context(ApiLimitError::Cache)?;

        if is_limit_reached && !self.limits.config.general().debug_disable_api_limits {
            Err(ApiLimitError::LimitReached.report())
        } else {
            Ok(())
        }
    }

    pub async fn post_reset_profile_paging(&self) -> Result<(), ApiLimitError> {
        self.check(|state, config| {
            state
                .post_reset_profile_paging
                .increment_and_check_is_limit_reached(
                    config
                        .limits_profile()
                        .profile_iterator_reset_daily_max_count,
                )
        })
        .await
    }

    pub async fn post_get_next_profile_page(&self) -> Result<(), ApiLimitError> {
        self.check(|state, config| {
            state
                .post_get_next_profile_page
                .increment_and_check_is_limit_reached(
                    config
                        .limits_profile()
                        .profile_iterator_next_page_daily_max_count,
                )
        })
        .await
    }

    pub async fn get_profile(&self) -> Result<(), ApiLimitError> {
        self.check(|state, config| {
            state.get_profile.increment_and_check_is_limit_reached(
                config.limits_profile().get_profile_daily_max_count,
            )
        })
        .await
    }
}

pub struct MediaApiLimits<'a> {
    limits: ApiLimits<'a>,
}

impl MediaApiLimits<'_> {
    async fn check(
        &self,
        check: impl FnOnce(&mut AllApiLimits, &Config) -> bool,
    ) -> Result<(), ApiLimitError> {
        let is_limit_reached = self
            .limits
            .cache
            .write_cache_common(self.limits.account_id, |e| {
                Ok(check(e.api_limits(), self.limits.config))
            })
            .await
            .change_context(ApiLimitError::Cache)?;

        if is_limit_reached && !self.limits.config.general().debug_disable_api_limits {
            Err(ApiLimitError::LimitReached.report())
        } else {
            Ok(())
        }
    }

    pub async fn get_profile_content_info(&self) -> Result<(), ApiLimitError> {
        self.check(|state, config| {
            state
                .get_profile_content_info
                .increment_and_check_is_limit_reached(
                    config
                        .limits_media()
                        .get_profile_content_info_daily_max_count,
                )
        })
        .await
    }
}
